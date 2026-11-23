//! Replay normalized events through CSL watches and write alert records.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};

use crate::stmt::{AbilityDecl, WatchDecl, WatchSequence};

pub type Grants<'a> = HashSet<(&'a str, &'a str)>;

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

struct RuleState<'a> {
    rule: &'a WatchDecl,
    alerts: &'a [String],
    timestamps: VecDeque<(i64, String)>,
    counts: HashMap<String, usize>,
}

impl RuleState<'_> {
    fn apply<'event>(&mut self, event: &'event InputEvent) -> Option<(&'event str, usize)> {
        // The trailing window is (timestamp - window, timestamp].
        if let Some(cutoff) = event.timestamp.checked_sub(i64::from(self.rule.window)) {
            while self
                .timestamps
                .front()
                .is_some_and(|(time, _)| *time <= cutoff)
            {
                let (_, group) = self
                    .timestamps
                    .pop_front()
                    .expect("checked the queue front");
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
        self.timestamps.push_back((event.timestamp, group.clone()));
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
        event: &InputEvent,
        line: usize,
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

        let mut emitted = false;
        for state in &mut self.rules {
            if let Some((group_key, count)) = state.apply(event) {
                for message in state.alerts {
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
                    serde_json::to_writer(&mut *output, &record)?;
                    writeln!(output)?;
                    emitted = true;
                }
            }
        }
        if emitted {
            output.flush()?;
        }
        Ok(())
    }
}

pub fn replay(
    watches: &[WatchDecl],
    abilities: &[AbilityDecl],
    grants: &Grants<'_>,
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
        let alerts = match &rule.sequence {
            WatchSequence::Inline(alerts) => alerts,
            WatchSequence::Named(reference) => {
                let name = reference.lexeme();
                let line = reference.line();
                let ability = abilities.get(name).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("[line {line}] Unknown ability '{name}'."),
                    )
                })?;
                if !grants.contains(&(ability.target.as_str(), ability.capability.as_str())) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "[line {line}] Ability '{name}' requires grant {:?} for target {:?}.",
                            ability.capability, ability.target
                        ),
                    ));
                }
                &ability.alerts
            }
        };
        sources
            .entry(&rule.source)
            .or_default()
            .rules
            .push(RuleState {
                rule,
                alerts,
                timestamps: VecDeque::new(),
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
        let event: InputEvent =
            serde_json::from_str(line).map_err(|error| input_error(number, error))?;
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
        state.replay_event(source, &event, number, &mut output)?;
    }
    Ok(())
}

fn input_error(line: usize, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Event input line {line}: {error}"),
    )
}
