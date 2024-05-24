use std::collections::HashMap;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

use crate::{JSONPathError, JSONPathParser, NodeList, Query};

#[pyclass]
pub struct Env {
    pub parser: JSONPathParser,
    pub function_register: Py<PyDict>,
    pub nothing: PyObject,
}

#[pymethods]
impl Env {
    #[new]
    #[pyo3(signature = (function_register, nothing, **options))]
    pub fn new<'py>(
        function_register: &Bound<'py, PyDict>,
        nothing: &Bound<'py, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let mut parser = JSONPathParser {
            index_range: ((-2_i64).pow(53) + 1..=2_i64.pow(53) - 1), // TODO: get from py env
            function_signatures: HashMap::new(),
            strict: option_strict(options)?,
        };

        // Derive function extension signatures from the function register
        for (k, v) in function_register.iter() {
            let name = k.extract::<String>().map_err(|_| {
                PyValueError::new_err("expected a function register with string keys")
            })?;

            let params = v
                .getattr("arg_types")
                .map_err(|_| {
                    PyValueError::new_err(format!(
                        "expected an `args_type` attribute on filter function `{}`",
                        k
                    ))
                })?
                .extract()
                .map_err(|_| {
                    PyValueError::new_err(format!(
                        "expected `args_type` to be a list of `ExpressionType`s on filter function `{}`",
                        k
                    ))
                })?;

            let returns = v
                .getattr("return_type")
                .map_err(|_| {
                    PyValueError::new_err(format!(
                        "expected a `return_type` attribute on filter function `{}`",
                        k
                    ))
                })?
                .extract()
                .map_err(|_| {
                    PyValueError::new_err(format!(
                        "expected `return_type` to be of `ExpressionType` on filter function `{}`",
                        k
                    ))
                })?;

            parser.add_function(&name, params, returns)
        }

        Ok(Env {
            parser,
            function_register: function_register.clone().unbind(),
            nothing: nothing.clone().unbind(),
        })
    }

    pub fn find<'py>(
        &self,
        query: &str,
        value: &Bound<'py, PyAny>,
    ) -> Result<NodeList<'py>, JSONPathError> {
        self.parser.parse(query)?.resolve(value, self)
    }

    pub fn compile(&self, query: &str) -> Result<Query, JSONPathError> {
        self.parser.parse(query)
    }

    pub fn query<'py>(
        &self,
        query: Query,
        value: &Bound<'py, PyAny>,
    ) -> Result<NodeList<'py>, JSONPathError> {
        query.resolve(value, self)
    }
}

fn option_strict(options: Option<&Bound<'_, PyDict>>) -> PyResult<bool> {
    match options {
        None => Ok(true),
        Some(py_dict) => Ok(py_dict
            .get_item("strict")?
            .map(|val| val.is_truthy())
            .unwrap_or(Ok(true))?),
    }
}
