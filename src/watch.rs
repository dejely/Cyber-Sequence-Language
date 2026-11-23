//! Replay normalized events through CSL watches and write alert records.

mod ranked;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead, Write};
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::stmt::{AbilityDecl, WatchDecl, WatchSequence};

pub struct Options<'a> {
    grants: HashSet<(&'a str, &'a str)>,
    rank: u8,
    policy: ranked::Policy,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            grants: HashSet::new(),
            rank: 9,
            policy: ranked::Policy::default(),
        }
    }
}

impl<'a> Options<'a> {
    pub fn parse(mut args: &'a [String]) -> Result<Self, String> {
        let mut options = Self::default();
        while let Some((flag, rest)) = args.split_first() {
            args = match (flag.as_str(), rest) {
                ("--grant", [target, capability, tail @ ..])
                    if crate::stmt::valid_grant_name(target)
                        && crate::stmt::valid_grant_name(capability) =>
                {
                    options
                        .grants
                        .insert((target.as_str(), capability.as_str()));
                    tail
                }
                ("--rank", [rank, tail @ ..]) => {
                    options.rank = rank
                        .parse::<u8>()
                        .ok()
                        .filter(|rank| *rank <= 9)
                        .ok_or("Expected --rank <0..9>.")?;
                    tail
                }
                ("--policy", [path, tail @ ..]) => {
                    options.policy = ranked::Policy::load(path)?;
                    tail
                }
                ("--rank", _) => return Err("Expected --rank <0..9>.".into()),
                ("--policy", _) => return Err("Expected --policy <file>.".into()),
                _ => return Err(
                    "Expected --grant <target> <capability> with nonblank target and capability."
                        .into(),
                ),
            };
        }
        Ok(options)
    }
}

#[derive(Deserialize)]
struct InputEvent {
    source: Option<String>,
    kind: String,
    timestamp: i64,
    #[serde(default)]
    fields: HashMap<String, String>,
}

#[derive(Serialize)]
struct AlertRecord<'a> {
    rule: &'a str,
    source: &'a str,
    kind: &'a str,
    group_key: &'a str,
    timestamp: i64,
    count: usize,
    step: &'static str,
    message: &'a str,
}

#[derive(Default)]
struct SourceState<'a> {
    last_timestamp: Option<i64>,
    rules: Vec<RuleState<'a>>,
}

enum ResolvedSequence<'a> {
    Inline(&'a [String]),
    Named(&'a AbilityDecl),
}

struct RuleState<'a> {
    rule: &'a WatchDecl,
    sequence: ResolvedSequence<'a>,
    events: VecDeque<WindowEntry>,
    counts: HashMap<String, usize>,
}

struct WindowEntry {
    timestamp: i64,
    group: String,
    event: Option<Rc<InputEvent>>,
}

impl RuleState<'_> {
    fn apply<'event>(&mut self, event: &'event Rc<InputEvent>) -> Option<(&'event str, usize)> {
        // The trailing window is (timestamp - window, timestamp].
        if let Some(cutoff) = event.timestamp.checked_sub(i64::from(self.rule.window)) {
            while self
                .events
                .front()
                .is_some_and(|entry| entry.timestamp <= cutoff)
            {
                let WindowEntry { group, .. } =
                    self.events.pop_front().expect("checked the queue front");
                let count = self
                    .counts
                    .get_mut(&group)
                    .expect("queued group has a count");
                *count -= 1;
                if *count == 0 {
                    self.counts.remove(&group);
                }
            }
        }

        if event.kind != self.rule.kind {
            return None;
        }
        let group = event
            .fields
            .get(&self.rule.group_field)
            .filter(|key| !key.is_empty())?;
        self.events.push_back(WindowEntry {
            timestamp: event.timestamp,
            group: group.clone(),
            event: matches!(self.sequence, ResolvedSequence::Named(_)).then(|| Rc::clone(event)),
        });
        let count = self.counts.entry(group.clone()).or_default();
        *count += 1;
        // Only a single event was added, so equality marks a threshold crossing.
        (*count == self.rule.threshold as usize).then_some((group, *count))
    }
}

impl SourceState<'_> {
    fn replay_event(
        &mut self,
        source: &str,
        event: &Rc<InputEvent>,
        line: usize,
        session: &Rc<RefCell<ranked::Session>>,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if let Some(previous) = self.last_timestamp
            && event.timestamp < previous
        {
            return Err(input_error(
                line,
                format_args!(
                    "source {source:?} received timestamp {} after {previous}",
                    event.timestamp
                ),
            ));
        }
        self.last_timestamp = Some(event.timestamp);

        for state in &mut self.rules {
            let Some((group_key, count)) = state.apply(event) else {
                continue;
            };
            match state.sequence {
                ResolvedSequence::Inline(alerts) => {
                    for message in alerts {
                        let record = AlertRecord {
                            rule: state.rule.name.lexeme(),
                            source,
                            kind: &event.kind,
                            group_key,
                            timestamp: event.timestamp,
                            count,
                            step: "alert",
                            message,
                        };
                        write_record(output, &record)?;
                    }
                }
                ResolvedSequence::Named(ability) => {
                    let summary = json!({
                        "rule": state.rule.name.lexeme(),
                        "source": source,
                        "kind": event.kind,
                        "group_key": group_key,
                        "timestamp": event.timestamp,
                        "count": count,
                        "target": ability.target
                    });
                    let matches = state
                        .events
                        .iter()
                        .filter(|entry| entry.group == group_key)
                        .filter_map(|entry| entry.event.as_deref())
                        .map(|event| {
                            json!({
                                "source": source, "kind": event.kind,
                                "timestamp": event.timestamp, "fields": event.fields
                            })
                        })
                        .collect::<Vec<_>>();
                    ranked::run(ability, summary, json!(matches), session, output)?;
                }
            }
        }
        Ok(())
    }
}

pub fn replay(
    watches: &[WatchDecl],
    abilities: &[AbilityDecl],
    options: Options<'_>,
    input: impl BufRead,
    mut output: impl Write,
) -> io::Result<()> {
    let abilities: HashMap<_, _> = abilities
        .iter()
        .map(|ability| (ability.name.lexeme(), ability))
        .collect();
    let mut sources: HashMap<&str, SourceState<'_>> = HashMap::new();
    for rule in watches {
        // Resolve and authorize every named sequence before reading any event.
        let sequence = match &rule.sequence {
            WatchSequence::Inline(alerts) => ResolvedSequence::Inline(alerts),
            WatchSequence::Named(reference) => {
                let name = reference.lexeme();
                let line = reference.line();
                let ability = abilities.get(name).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("[line {line}] Unknown ability '{name}'."),
                    )
                })?;
                if !options
                    .grants
                    .contains(&(ability.target.as_str(), ability.capability.as_str()))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "[line {line}] Ability '{name}' requires grant {:?} for target {:?}.",
                            ability.capability, ability.target
                        ),
                    ));
                }
                ResolvedSequence::Named(ability)
            }
        };
        sources
            .entry(&rule.source)
            .or_default()
            .rules
            .push(RuleState {
                rule,
                sequence,
                events: VecDeque::new(),
                counts: HashMap::new(),
            });
    }
    if sources.is_empty() {
        return Ok(());
    }
    let default_source = if sources.len() == 1 {
        sources.keys().next().copied()
    } else {
        None
    };

    let session = Rc::new(RefCell::new(ranked::Session::new(
        options.rank,
        options.policy,
    )));
    for (index, line) in input.lines().enumerate() {
        let number = index + 1;
        let line = line.map_err(|error| input_error(number, error))?;
        let line = line.trim_start_matches([' ', '\t', '\r', '\n']);
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('{') {
            return Err(input_error(number, "expected a JSON event object"));
        }
        let event: Rc<InputEvent> =
            Rc::new(serde_json::from_str(line).map_err(|error| input_error(number, error))?);
        let source = event.source.as_deref().or(default_source).ok_or_else(|| {
            input_error(
                number,
                "an event must name its source when multiple sources are watched",
            )
        })?;
        let state = sources
            .get_mut(source)
            .ok_or_else(|| input_error(number, format_args!("unknown event source {source:?}")))?;
        if event.kind.is_empty() {
            return Err(input_error(number, "event kind must be nonempty"));
        }
        state.replay_event(source, &event, number, &session, &mut output)?;
    }
    Ok(())
}

fn write_record(output: &mut impl Write, record: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *output, record)?;
    writeln!(output)?;
    output.flush()
}

fn input_error(line: usize, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Event input line {line}: {error}"),
    )
}
