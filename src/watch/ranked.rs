//! Trigger-local native functions and replay-session simulation records.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::{self, Write};
use std::rc::Rc;

use serde::Deserialize;
use serde_json::{Value as Json, json};

use crate::evaluator::{Interpreter, RuntimeError};
use crate::function::{Callable, NativeFunction};
use crate::stmt::{AbilityDecl, AbilityStep};
use crate::token::Token;
use crate::value::Value;

use super::write_record;

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Policy {
    targets: BTreeMap<String, Target>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Target {
    metadata: BTreeMap<String, Json>,
    containment_impact: BTreeMap<String, Json>,
    responses: BTreeMap<String, Vec<String>>,
    default_response: Option<String>,
}

impl Policy {
    pub(super) fn load(path: &str) -> Result<Self, String> {
        let file =
            std::fs::File::open(path).map_err(|error| format!("Policy {path:?}: {error}"))?;
        let policy: Self =
            serde_json::from_reader(file).map_err(|error| format!("Policy {path:?}: {error}"))?;
        for (name, target) in &policy.targets {
            if name.trim().is_empty() || target.responses.keys().any(|name| name.trim().is_empty())
            {
                return Err(format!(
                    "Policy {path:?}: target and response names must be nonblank."
                ));
            }
            if let Some(response) = &target.default_response
                && !target.responses.contains_key(response)
            {
                return Err(format!(
                    "Policy {path:?}: target {name:?} has an unapproved default response {response:?}."
                ));
            }
        }
        Ok(policy)
    }
}

pub(super) struct Session {
    rank: u8,
    policy: Policy,
    records: HashMap<String, Json>,
    next_id: u64,
}

impl Session {
    pub(super) fn new(rank: u8, policy: Policy) -> Self {
        Self {
            rank,
            policy,
            records: HashMap::new(),
            next_id: 1,
        }
    }
}

struct Activation {
    session: Rc<RefCell<Session>>,
    summary: Json,
    matches: Json,
    target: String,
    output: Vec<Json>,
    containment_reason: Option<String>,
    containment_id: Option<String>,
}

#[derive(Clone, Copy)]
struct Operation {
    name: &'static str,
    rank: u8,
    arity: usize,
}

const FUNCTIONS: &[(&str, u8, usize)] = &[
    ("inspect", 6, 0),
    ("triggerSummary", 6, 0),
    ("matchedEvents", 6, 0),
    ("eventTimeline", 6, 0),
    ("exportReport", 6, 0),
    ("targetInfo", 3, 0),
    ("isolate", 3, 0),
    ("previewImpact", 3, 0),
    ("recordContainmentReason", 3, 1),
    ("cancelContainmentIntent", 3, 1),
    ("listApprovedResponses", 2, 0),
    ("previewResponse", 2, 1),
    ("execute", 2, 0),
    ("responseResult", 2, 1),
    ("cancelResponseIntent", 2, 1),
    ("openIncident", 1, 0),
    ("attachEvidence", 1, 2),
    ("requestReview", 1, 1),
    ("escalateIncident", 1, 1),
    ("closeIncident", 1, 1),
];

struct RankedFunction {
    operation: Operation,
    activation: Rc<RefCell<Activation>>,
}

impl Callable for RankedFunction {
    fn arity(&self) -> usize {
        self.operation.arity
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        args: Vec<Value>,
        site: &Token,
    ) -> Result<Value, RuntimeError> {
        self.activation
            .borrow_mut()
            .invoke(self.operation, &args, site, false)
            .map(Value::from_json)
    }
}

impl fmt::Display for RankedFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.operation.name)
    }
}

pub(super) fn run(
    ability: &AbilityDecl,
    summary: Json,
    matches: Json,
    session: &Rc<RefCell<Session>>,
    output: &mut impl Write,
) -> io::Result<()> {
    let activation = Rc::new(RefCell::new(Activation {
        session: Rc::clone(session),
        summary,
        matches,
        target: ability.target.clone(),
        output: Vec::new(),
        containment_reason: None,
        containment_id: None,
    }));
    let mut interpreter = Interpreter::default();
    let scope = interpreter.child_scope();
    {
        let mut bindings = scope.borrow_mut();
        bindings.define("get", Value::Callable(Rc::new(NativeFunction::Get)));
        for &(name, rank, arity) in FUNCTIONS {
            let operation = Operation { name, rank, arity };
            bindings.define(
                operation.name,
                Value::Callable(Rc::new(RankedFunction {
                    operation,
                    activation: Rc::clone(&activation),
                })),
            );
        }
    }
    for step in &ability.steps {
        let result = match step {
            AbilityStep::Alert(message) => {
                let mut activation = activation.borrow_mut();
                let mut record = activation.summary.clone();
                record
                    .as_object_mut()
                    .expect("trigger summary object")
                    .remove("target");
                record["step"] = json!("alert");
                record["message"] = json!(message);
                activation.output.push(record);
                Ok(())
            }
            AbilityStep::Keyword(token) => {
                let &(name, rank, arity) = FUNCTIONS
                    .iter()
                    .find(|(name, ..)| *name == token.lexeme())
                    .expect("known sequence keyword");
                activation
                    .borrow_mut()
                    .invoke(Operation { name, rank, arity }, &[], token, true)
                    .map(|_| ())
            }
            AbilityStep::Statement(statement) => interpreter
                .execute_block(std::slice::from_ref(statement), Rc::clone(&scope))
                .map(|_| ()),
        };
        for record in activation.borrow_mut().output.drain(..) {
            write_record(output, &record)?;
        }
        result.map_err(|error| io::Error::other(error.to_string()))?;
    }
    Ok(())
}

// IDs are session-local. Every read or mutation checks both target and record kind.
fn scoped_record<'a>(
    records: &'a mut HashMap<String, Json>,
    id: &str,
    target: &str,
    kind: &str,
) -> Option<&'a mut Json> {
    if !id
        .strip_prefix(kind)
        .is_some_and(|suffix| suffix.starts_with('-'))
    {
        return None;
    }
    records
        .get_mut(id)
        .filter(|record| record["target"] == target)
}

impl Activation {
    fn invoke(
        &mut self,
        operation: Operation,
        args: &[Value],
        site: &Token,
        legacy_keyword: bool,
    ) -> Result<Json, RuntimeError> {
        let session = Rc::clone(&self.session);
        let mut state = session.borrow_mut();
        if state.rank > operation.rank {
            return Err(RuntimeError::new(
                site,
                format!(
                    "{} requires Sequence {} or a stronger rank.",
                    operation.name, operation.rank
                ),
            ));
        }

        let first = || match &args[0] {
            Value::String(text) if !text.trim().is_empty() => Ok(text.as_str()),
            _ => Err(RuntimeError::new(site, "Expected a nonblank string.")),
        };
        let target = state.policy.targets.get(&self.target);
        Ok(match operation.name {
            "inspect" => self.emit("inspection", self.summary.clone()),
            "triggerSummary" => self.summary.clone(),
            // Replay already processes each source in timestamp order, including ties.
            "matchedEvents" | "eventTimeline" => self.matches.clone(),
            "exportReport" => self.emit(
                "report",
                json!({
                    "target": self.target, "trigger": self.summary, "events": self.matches
                }),
            ),
            "targetInfo" => json!({
                "target": self.target, "metadata": target.map(|target| &target.metadata)
            }),
            "previewImpact" => json!({
                "target": self.target, "impact": target.map(|target| &target.containment_impact)
            }),
            "listApprovedResponses" => json!(
                target
                    .into_iter()
                    .flat_map(|target| target.responses.keys())
                    .collect::<Vec<_>>()
            ),
            "previewResponse" => {
                let name = first()?;
                let steps = target
                    .and_then(|target| target.responses.get(name))
                    .ok_or_else(|| {
                        RuntimeError::new(
                            site,
                            format!("Response {name:?} is not approved for this target."),
                        )
                    })?;
                json!({ "target": self.target, "name": name, "steps": steps })
            }
            "isolate" => {
                let mut record = self.summary.clone();
                record["status"] = json!("pending");
                record["result"] = json!("simulated");
                record["reason"] = json!(self.containment_reason);
                let id = self.insert(&mut state, "containment", "simulated_isolate", record);
                self.containment_id = Some(id.clone());
                json!(id)
            }
            "recordContainmentReason" => {
                let reason = first()?;
                self.containment_reason = Some(reason.to_owned());
                if let Some(id) = &self.containment_id
                    && let Some(record) =
                        scoped_record(&mut state.records, id, &self.target, "containment")
                    && record["status"] == "pending"
                {
                    record["reason"] = json!(reason);
                    self.emit("containment_updated", record.clone());
                }
                Json::Null
            }
            "execute" => {
                let response = target.and_then(|target| {
                    target
                        .default_response
                        .as_ref()
                        .map(|name| json!({ "name": name, "steps": target.responses[name] }))
                });
                if response.is_none() && !legacy_keyword {
                    return Err(RuntimeError::new(
                        site,
                        "This target has no approved default response.",
                    ));
                }
                let mut record = self.summary.clone();
                record["status"] = json!("pending");
                record["result"] = json!("simulated");
                record["response"] = json!(response);
                json!(self.insert(&mut state, "response", "simulated_execute", record))
            }
            "responseResult" => {
                scoped_record(&mut state.records, first()?, &self.target, "response")
                    .map_or(Json::Null, |record| record.clone())
            }
            "cancelContainmentIntent" => json!(self.cancel(&mut state, first()?, "containment")),
            "cancelResponseIntent" => json!(self.cancel(&mut state, first()?, "response")),
            "openIncident" => {
                let mut record = self.summary.clone();
                record["status"] = json!("open");
                record["evidence"] = json!([]);
                json!(self.insert(&mut state, "incident", "incident_opened", record))
            }
            "attachEvidence" => {
                let evidence = args[1]
                    .to_json()
                    .ok_or_else(|| RuntimeError::new(site, "Evidence must be data."))?;
                json!(self.update_incident(&mut state, first()?, None, Some(evidence)))
            }
            "requestReview" => {
                json!(self.update_incident(&mut state, first()?, Some("in_review"), None))
            }
            "escalateIncident" => {
                json!(self.update_incident(&mut state, first()?, Some("escalated"), None))
            }
            "closeIncident" => {
                json!(self.update_incident(&mut state, first()?, Some("closed"), None))
            }
            _ => unreachable!("registered ranked operation"),
        })
    }

    fn emit(&mut self, step: &str, mut record: Json) -> Json {
        record["step"] = json!(step);
        self.output.push(record.clone());
        record
    }

    fn insert(&mut self, state: &mut Session, kind: &str, step: &str, mut record: Json) -> String {
        let id = format!("{kind}-{}", state.next_id);
        state.next_id += 1;
        record["id"] = json!(id);
        state.records.insert(id.clone(), self.emit(step, record));
        id
    }

    fn cancel(&mut self, state: &mut Session, id: &str, kind: &str) -> bool {
        let Some(record) = scoped_record(&mut state.records, id, &self.target, kind)
            .filter(|record| record["status"] == "pending")
        else {
            return false;
        };
        record["status"] = json!("canceled");
        self.emit("intent_canceled", record.clone());
        true
    }

    fn update_incident(
        &mut self,
        state: &mut Session,
        id: &str,
        status: Option<&str>,
        evidence: Option<Json>,
    ) -> bool {
        let Some(record) = scoped_record(&mut state.records, id, &self.target, "incident")
            .filter(|record| record["status"] != "closed")
        else {
            return false;
        };
        if let Some(status) = status {
            let allowed = match status {
                "in_review" => record["status"] == "open",
                "escalated" => record["status"] == "open" || record["status"] == "in_review",
                "closed" => true,
                _ => unreachable!("known incident status"),
            };
            if !allowed {
                return false;
            }
            record["status"] = json!(status);
        }
        if let Some(evidence) = evidence {
            record["evidence"]
                .as_array_mut()
                .expect("incident evidence array")
                .push(evidence);
        }
        self.emit("incident_updated", record.clone());
        true
    }
}
