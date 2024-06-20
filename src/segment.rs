use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::node::{Location, PathElement, Value};
use crate::selector::Selector;
use crate::{NodeList, QueryContext};

#[pyclass]
#[derive(Debug, Clone)]
pub enum Segment {
    Child { selectors: Vec<Selector> },
    Recursive { selectors: Vec<Selector> },
    Eoi {},
}

impl Segment {
    pub fn resolve(&self, nodes: NodeList, context: &QueryContext) -> NodeList {
        match self {
            Segment::Child { selectors } => nodes
                .into_iter()
                .flat_map(|node| {
                    selectors.iter().map(move |s| {
                        s.resolve(node.value.bind(context.root.py()), &node.location, context)
                    })
                })
                .flatten()
                .collect(),
            Segment::Recursive { selectors } => nodes
                .into_iter()
                .flat_map(move |node| {
                    self.visit(
                        node.value.bind(context.root.py()),
                        node.location,
                        selectors,
                        context,
                    )
                })
                .collect(),
            Segment::Eoi {} => nodes,
        }
    }

    fn visit(
        &self,
        value: &Value<'_>,
        location: Location,
        selectors: &Vec<Selector>,
        context: &QueryContext,
    ) -> NodeList {
        let mut nodes: NodeList = selectors
            .iter()
            .flat_map(|s| s.resolve(value, &location, context))
            .collect();

        nodes.append(&mut self.descend(value, &location, selectors, context));
        nodes
    }

    fn descend(
        &self,
        value: &Value<'_>,
        location: &Location,
        selectors: &Vec<Selector>,
        context: &QueryContext,
    ) -> NodeList {
        if let Ok(list) = value.downcast::<PyList>() {
            list.iter()
                .enumerate()
                .flat_map(|(i, v)| {
                    self.visit(
                        &v,
                        location.append(PathElement::Index(i)),
                        selectors,
                        context,
                    )
                })
                .collect()
        } else if let Ok(dict) = value.downcast::<PyDict>() {
            dict.iter()
                .flat_map(|(k, v)| {
                    self.visit(
                        &v,
                        location.append(PathElement::Name(k.extract().unwrap())),
                        selectors,
                        context,
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }
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
            Segment::Eoi {} => Ok(()),
        }
    }
}

#[pymethods]
impl Segment {
    fn __repr__(&self) -> String {
        match self {
            Segment::Child { .. } => format!("<jpq.Segment.Child `{}`>", self),
            Segment::Recursive { .. } => format!("<jpq.Segment.Recursive `{}`>", self),
            Segment::Eoi {} => String::from(""),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}
