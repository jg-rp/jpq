use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::selector::Selector;
use crate::{JSONPathError, Node, NodeList, QueryContext};

#[pyclass]
#[derive(Debug, Clone)]
pub enum Segment {
    Child {
        span: (usize, usize),
        selectors: Vec<Selector>,
    },
    Recursive {
        span: (usize, usize),
        selectors: Vec<Selector>,
    },
}

impl Segment {
    pub fn resolve<'py>(
        &self,
        nodes: NodeList<'py>,
        context: &QueryContext,
    ) -> Result<NodeList<'py>, JSONPathError> {
        match self {
            Segment::Child { selectors, .. } => {
                let mut _nodes: NodeList = Vec::new();
                for node in nodes.iter() {
                    for selector in selectors {
                        _nodes.extend(selector.resolve(node, context)?);
                    }
                }
                Ok(_nodes)
            }
            Segment::Recursive { selectors, .. } => {
                let mut _nodes: NodeList = Vec::new();
                for node in nodes {
                    for _node in visit(node).iter() {
                        for selector in selectors {
                            _nodes.extend(selector.resolve(_node, context)?);
                        }
                    }
                }
                Ok(_nodes)
            }
        }
    }
}

fn visit(node: Node<'_>) -> NodeList<'_> {
    let value = &node.0.clone();
    let loc = node.1.to_owned();
    let mut nodes: NodeList = vec![node.clone()];

    if let Ok(list) = value.downcast::<PyList>() {
        for (i, element) in list.iter().enumerate() {
            let _node = (element, format!("{}[{}]", loc, i), i.into_py(value.py()));
            let children = visit(_node);
            nodes.extend(children)
        }
    } else if let Ok(dict) = value.downcast::<PyDict>() {
        for (key, val) in dict.iter() {
            let _node = (val, format!("{}['{}']", loc, key), key.unbind());
            let children = visit(_node);
            nodes.extend(children);
        }
    }

    nodes
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Segment::Child { selectors, .. } => {
                write!(
                    f,
                    "[{}]",
                    selectors
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Segment::Recursive { selectors, .. } => {
                write!(
                    f,
                    "..[{}]",
                    selectors
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

#[pymethods]
impl Segment {
    fn __repr__(&self) -> String {
        match self {
            Segment::Child { .. } => format!("<jpq.Segment.Child `{}`>", self),
            Segment::Recursive { .. } => format!("<jpq.Segment.Recursive `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}
