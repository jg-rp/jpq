use std::fmt;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[derive(Debug)]
pub enum JSONPathErrorType {
    LexerError,
    SyntaxError,
    TypeError,
    NameError,
    ExtError,
}

#[derive(Debug)]
pub struct JSONPathError {
    pub kind: JSONPathErrorType,
    pub msg: String,
}

impl JSONPathError {
    pub fn new(error: JSONPathErrorType, msg: String) -> Self {
        Self { kind: error, msg }
    }

    pub fn syntax(msg: String) -> Self {
        Self {
            kind: JSONPathErrorType::SyntaxError,
            msg,
        }
    }

    pub fn typ(msg: String) -> Self {
        Self {
            kind: JSONPathErrorType::TypeError,
            msg,
        }
    }

    pub fn name(msg: String) -> Self {
        Self {
            kind: JSONPathErrorType::NameError,
            msg,
        }
    }

    pub fn ext(msg: String) -> Self {
        Self {
            kind: JSONPathErrorType::ExtError,
            msg,
        }
    }
}

impl std::error::Error for JSONPathError {}

create_exception!(
    jpq,
    PyJSONPathError,
    PyException,
    "Base exception for all JSONPath errors."
);

create_exception!(
    jpq,
    JSONPathTypeError,
    PyJSONPathError,
    "JSONPath type error."
);

create_exception!(
    jpq,
    JSONPathSyntaxError,
    PyJSONPathError,
    "JSONPath syntax error."
);

create_exception!(
    jpq,
    JSONPathNameError,
    PyJSONPathError,
    "JSONPath name error."
);

create_exception!(
    jpq,
    JSONPathExtensionError,
    PyJSONPathError,
    "JSONPath function extension error."
);

impl std::convert::From<JSONPathError> for PyErr {
    fn from(err: JSONPathError) -> Self {
        use JSONPathErrorType::*;
        match err.kind {
            // TODO: improve error messages
            TypeError => JSONPathTypeError::new_err(err.to_string()),
            SyntaxError => JSONPathSyntaxError::new_err(err.to_string()),
            NameError => JSONPathNameError::new_err(err.to_string()),
            ExtError => JSONPathExtensionError::new_err(err.to_string()),
            _ => PyJSONPathError::new_err(err.to_string()),
        }
    }
}

impl fmt::Display for JSONPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
