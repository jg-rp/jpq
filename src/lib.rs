pub mod environment;
pub mod errors;
pub mod filter;
pub mod lexer;
pub mod parser;
pub mod query;
pub mod segment;
pub mod selector;
pub mod token;

use std::collections::HashMap;

pub use errors::JSONPathError;
pub use errors::JSONPathErrorType;
pub use parser::Parser;
pub use query::Query;

use pyo3::prelude::*;

pub type Node<'py> = (Bound<'py, PyAny>, String); // TODO: do we need location? for "keys"?
pub type NodeList<'py> = Vec<Node<'py>>;

pub struct QueryContext<'py> {
    env: &'py environment::Env,
    root: Bound<'py, PyAny>,
}

pub struct FilterContext<'py> {
    env: &'py environment::Env,
    root: Bound<'py, PyAny>,
    current: Bound<'py, PyAny>,
    current_key: Option<Bound<'py, PyAny>>,
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum ExpressionType {
    Logical,
    Nodes,
    Value,
}

pub struct FunctionSignature {
    pub param_types: Vec<ExpressionType>,
    pub return_type: ExpressionType,
}

pub fn standard_functions() -> HashMap<String, FunctionSignature> {
    let mut functions = HashMap::new();

    functions.insert(
        "count".to_owned(),
        FunctionSignature {
            param_types: vec![ExpressionType::Nodes],
            return_type: ExpressionType::Value,
        },
    );

    functions.insert(
        "length".to_owned(),
        FunctionSignature {
            param_types: vec![ExpressionType::Value],
            return_type: ExpressionType::Value,
        },
    );

    functions.insert(
        "match".to_owned(),
        FunctionSignature {
            param_types: vec![ExpressionType::Value, ExpressionType::Value],
            return_type: ExpressionType::Logical,
        },
    );

    functions.insert(
        "search".to_owned(),
        FunctionSignature {
            param_types: vec![ExpressionType::Value, ExpressionType::Value],
            return_type: ExpressionType::Logical,
        },
    );

    functions.insert(
        "value".to_owned(),
        FunctionSignature {
            param_types: vec![ExpressionType::Nodes],
            return_type: ExpressionType::Value,
        },
    );

    functions
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
    m.add_class::<ExpressionType>()?;
    m.add_class::<segment::Segment>()?;
    m.add_class::<selector::Selector>()?;
    m.add_class::<filter::LogicalOp>()?;
    m.add_class::<filter::ComparisonOp>()?;
    m.add_class::<filter::FilterExpression>()?;
    m.add_class::<query::Query>()?;
    m.add_class::<environment::Env>()?;
    Ok(())
}
