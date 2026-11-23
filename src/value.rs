use std::fmt;
use std::rc::Rc;

use crate::function::Callable;
use serde_json::{Value as Json, json};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Data(Rc<Json>),
    /// Functions are ordinary values: storable, passable, and returnable.
    Callable(Rc<dyn Callable>),
}

impl Value {
    pub(crate) fn from_json(value: Json) -> Self {
        match value {
            Json::Null => Self::Nil,
            Json::Bool(value) => Self::Boolean(value),
            Json::Number(value) => Self::Number(value.as_f64().expect("JSON number fits an f64")),
            Json::String(value) => Self::String(value),
            Json::Array(_) | Json::Object(_) => Self::Data(Rc::new(value)),
        }
    }

    pub(crate) fn to_json(&self) -> Option<Json> {
        Some(match self {
            Self::Nil => Json::Null,
            Self::Boolean(value) => json!(value),
            Self::Number(value) if value.is_finite() => json!(value),
            Self::String(value) => json!(value),
            Self::Data(value) => value.as_ref().clone(),
            Self::Number(_) | Self::Callable(_) => return None,
        })
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Boolean(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Nil => write!(f, "nil"),
            Self::Data(value) => write!(f, "{value}"),
            Self::Callable(function) => write!(f, "{function}"),
        }
    }
}
