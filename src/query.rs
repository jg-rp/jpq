use std::fmt;

use pyo3::prelude::*;

use crate::environment::Env;
use crate::segment::Segment;
use crate::selector::Selector;
use crate::{JSONPathError, NodeList, QueryContext};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Query {
    #[pyo3(get)]
    pub segments: Vec<Segment>,
}

impl<'py> pyo3::FromPyObject<'py> for Box<Query> {
    fn extract(ob: &'py PyAny) -> PyResult<Self> {
        ob.extract::<Query>().map(Box::new)
    }
}

impl pyo3::IntoPy<pyo3::PyObject> for Box<Query> {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        (*self).into_py(py)
    }
}

impl Query {
    pub fn new(segments: Vec<Segment>) -> Self {
        Query { segments }
    }

    // Apply this query to Python object `value` using the function register from `env`.
    pub fn resolve<'py>(
        &self,
        value: &Bound<'py, PyAny>,
        env: &Env,
    ) -> Result<NodeList<'py>, JSONPathError> {
        let root_node = vec![(value.clone(), "$".to_owned(), value.py().None())];
        let context = QueryContext {
            env,
            root: value.clone(),
        };

        self.segments
            .iter()
            .try_fold(root_node, |nodes, segment| segment.resolve(nodes, &context))
    }

    // Returns `true` if this query has no segments, or `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    // Returns `true` if this query can resolve to at most one node, or `false` otherwise.
    pub fn is_singular(&self) -> bool {
        self.segments.iter().all(|segment| {
            if let Segment::Child { selectors, .. } = segment {
                return selectors.len() == 1
                    && selectors.first().is_some_and(|selector| {
                        matches!(selector, Selector::Name { .. } | Selector::Index { .. })
                    });
            }
            false
        })
    }
}

#[pymethods]
impl Query {
    fn __repr__(&self) -> String {
        format!("<jpq.Query \"{}\">", self)
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "${}",
            self.segments
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("")
        )
    }
}
