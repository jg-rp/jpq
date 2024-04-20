//! A JSONPath expression parser, producing a JSON implementation agnostic
//! abstract syntax tree, following the JSONPath model described in RFC 9535.
//!
//! ## Standard queries
//!
//! To parse a JSONPath expression that is limited to standard [function extensions],
//! use [`Query::standard`].
//!
//! ```
//! use jpq::{errors::JSONPathError, Query};
//!
//! fn main() -> Result<(), JSONPathError> {
//!     let q = Query::standard("$..foo[0]")?;
//!     println!("{:#?}", q);
//!     Ok(())
//! }
//! ```
//!
//! Debug output from the example above shows this syntax tree:
//!
//! ```text
//! Query {
//!     segments: [
//!         Recursive {
//!             span: (
//!                 1,
//!                 3,
//!             ),
//!             selectors: [
//!                 Name {
//!                     span: (
//!                         3,
//!                         6,
//!                     ),
//!                     name: "foo",
//!                 },
//!             ],
//!         },
//!         Child {
//!             span: (
//!                 6,
//!                 7,
//!             ),
//!             selectors: [
//!                 Index {
//!                     span: (
//!                         7,
//!                         8,
//!                     ),
//!                     index: 0,
//!                 },
//!             ],
//!         },
//!     ],
//! }
//! ```
//!
//! ## Function extensions
//!
//! Register [function extensions] with a new [`Parser`] by calling [`Parser::add_function`],
//! then use [`Parser::parse`] to create a new [`Query`].
//!
//! ```
//! use jpq::{errors::JSONPathError, ExpressionType, Parser};
//!
//! fn main() -> Result<(), JSONPathError> {
//!     let mut parser = Parser::new();
//!
//!     parser.add_function(
//!         "foo",
//!         vec![ExpressionType::Value, ExpressionType::Nodes],
//!         ExpressionType::Logical,
//!     );
//!
//!     let q = parser.parse("$.some[?foo('7', @.thing)][1, 4]")?;
//!     println!("{:?}", q);
//!     Ok(())
//! }
//! ```
//!
//! Note that a [`Query`] is displayed in its canonical form when printed.
//!
//! ```text
//! $['some'][?foo("7", @['thing'])][1, 4]
//! ```
//!
//! Without registering a signature for `foo`, we would get a [`JSONPathError`] with
//! `kind` set to [`JSONPathErrorType::NameError`].
//!
//! ```text
//! Error: JSONPathError { kind: NameError, msg: "unknown function `foo`", span: (8, 11) }
//! ```
//!
//! [function extensions]: https://datatracker.ietf.org/doc/html/rfc9535#name-function-extensions
pub mod errors;
pub mod lexer;
pub mod parser;
pub mod query;
pub mod token;

pub use errors::JSONPathError;
pub use errors::JSONPathErrorType;
pub use parser::standard_functions;
pub use parser::ExpressionType;
pub use parser::FunctionSignature;
pub use parser::Parser;
pub use query::Query;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

impl std::convert::From<JSONPathError> for PyErr {
    fn from(err: JSONPathError) -> Self {
        // TODO: custom python error class
        PyValueError::new_err(err.to_string()) // TODO: include span
    }
}

#[pyfunction]
fn parse(query: &str) -> Result<Query, JSONPathError> {
    Query::standard(query)
}

// TODO: pyfunction for Parser with function extensions
// TODO: or, more likely, a pyclass wrapping Parser.add_function and Parser.parse

#[pymodule]
#[pyo3(name = "jpq")]
fn jpq_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_class::<query::Segment>()?;
    m.add_class::<query::Selector>()?;
    m.add_class::<query::LogicalOperator>()?;
    m.add_class::<query::ComparisonOperator>()?;
    m.add_class::<query::FilterExpression>()?;
    m.add_class::<query::Query>()?;
    Ok(())
}
