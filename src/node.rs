use std::{collections::VecDeque, iter};

use pyo3::prelude::*;
use pyo3::{Bound, PyAny, PyObject};

use crate::conslist::ConsList;

pub type Location = ConsList<PathElement>;
pub type NodeList = Vec<Node>;
pub type Value<'py> = Bound<'py, PyAny>;

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Node {
    #[pyo3(get)]
    pub value: PyObject,
    pub location: Location,
}

/// An array element index or object member name in a Node's location.
#[derive(Debug, Clone)]
pub enum PathElement {
    Index(usize),
    Name(String),
}

impl Node {
    pub fn new_array_element<'py>(
        value: Bound<'py, PyAny>,
        location: &Location,
        index: usize,
    ) -> Self {
        Node {
            value: value.unbind(),
            location: location.append(PathElement::Index(index)),
        }
    }

    pub fn new_object_member<'py>(
        value: Bound<'py, PyAny>,
        location: &Location,
        name: String,
    ) -> Self {
        Node {
            value: value.unbind(),
            location: location.append(PathElement::Name(name)),
        }
    }
}

#[pymethods]
impl Node {
    /// The location of this node's value in the query argument as a normalized path.
    pub fn path(&self) -> String {
        iter::once(String::from("$"))
            .chain(
                VecDeque::from_iter(self.location.iter().map(|e| match e {
                    PathElement::Index(i) => format!("[{}]", i),
                    PathElement::Name(s) => format!("['{}']", s),
                }))
                .into_iter()
                .rev(),
            )
            .collect::<Vec<String>>()
            .join("")
    }

    // TODO: key()
}
