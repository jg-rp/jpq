use pyo3::{prelude::*, types::PyDict};

use crate::{query::NodeList, JSONPathError, Parser, Query};

#[pyclass]
pub struct Env {
    pub parser: Parser,
    pub function_register: Py<PyDict>,
    pub nothing: PyObject,
}

#[pymethods]
impl Env {
    #[new]
    pub fn new<'py>(function_register: &Bound<'py, PyDict>, nothing: &Bound<'py, PyAny>) -> Self {
        // TODO: derive function types for parser from function register
        Env {
            parser: Parser::new(),
            function_register: function_register.clone().unbind(),
            nothing: nothing.clone().unbind(),
        }
    }

    pub fn find<'py>(
        &self,
        query: &str,
        value: &Bound<'py, PyAny>,
    ) -> Result<NodeList<'py>, JSONPathError> {
        let query = self.parser.parse(query)?;
        // TODO: propagate error
        Ok(query.resolve(value, self))
    }

    pub fn compile(&self, query: &str) -> Result<Query, JSONPathError> {
        // TODO: Need to pair this result with an env python-side
        self.parser.parse(query)
    }

    pub fn query<'py>(
        &self,
        query: Query,
        value: &Bound<'py, PyAny>,
    ) -> Result<NodeList<'py>, JSONPathError> {
        // TODO: propagate error
        Ok(query.resolve(value, self))
    }
}
