pub mod environment;
pub mod errors;
pub mod filter;
pub mod lexer;
pub mod parser;
pub mod query;
pub mod segment;
pub mod selector;
pub mod token;

pub use errors::JSONPathError;
pub use errors::JSONPathErrorType;
pub use parser::standard_functions;
pub use parser::ExpressionType;
pub use parser::FunctionSignature;
pub use parser::Parser;
pub use query::Query;

use pyo3::prelude::*;

pub type Node<'py> = (Bound<'py, PyAny>, String);
pub type NodeList<'py> = Vec<Node<'py>>;

pub struct QueryContext<'py> {
    env: &'py environment::Env,
    root: Bound<'py, PyAny>,
}

pub struct FilterContext<'py> {
    env: &'py environment::Env,
    root: Bound<'py, PyAny>,
    current: Bound<'py, PyAny>,
}

#[pymodule]
#[pyo3(name = "jpq")]
fn jpq_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add(
        "PyJSONPathError",
        m.py().get_type_bound::<errors::PyJSONPathError>(),
    )?;
    m.add(
        "JSONPathTypeError",
        m.py().get_type_bound::<errors::JSONPathTypeError>(),
    )?;
    m.add(
        "JSONPathSyntaxError",
        m.py().get_type_bound::<errors::JSONPathSyntaxError>(),
    )?;
    m.add(
        "JSONPathNameError",
        m.py().get_type_bound::<errors::JSONPathNameError>(),
    )?;
    m.add(
        "JSONPathExtensionError",
        m.py().get_type_bound::<errors::JSONPathExtensionError>(),
    )?;
    m.add_class::<segment::Segment>()?;
    m.add_class::<selector::Selector>()?;
    m.add_class::<filter::LogicalOp>()?;
    m.add_class::<filter::ComparisonOp>()?;
    m.add_class::<filter::FilterExpression>()?;
    m.add_class::<query::Query>()?;
    m.add_class::<environment::Env>()?;
    Ok(())
}
