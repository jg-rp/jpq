use std::cmp;
use std::fmt::{self, Write};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::filter::{is_truthy, FilterExpression};
use crate::node::{Location, Value};
use crate::{Node, NodeList, QueryContext};

#[pyclass]
#[derive(Debug, Clone)]
pub enum Selector {
    Name {
        name: String,
    },
    Index {
        index: i64,
    },
    Slice {
        start: Option<i64>,
        stop: Option<i64>,
        step: Option<i64>,
    },
    Wild {},
    Filter {
        expression: Box<FilterExpression>,
    },
}

impl Selector {
    pub fn resolve<'py>(
        &self,
        value: &Value<'py>,
        location: &Location,
        context: &QueryContext,
    ) -> NodeList {
        match self {
            Selector::Name { name, .. } => {
                if let Ok(v) = value.get_item(name) {
                    vec![Node::new_object_member(v, location, name.to_owned())]
                } else {
                    Vec::new()
                }
            }
            Selector::Index { index, .. } => {
                // We don't want to index Python strings or dicts with integer keys
                if value.is_instance_of::<PyList>() {
                    let norm = norm_index(*index, value.len().unwrap());
                    if let Ok(v) = value.get_item(norm) {
                        vec![Node::new_array_element(v, location, norm)]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Selector::Slice {
                start, stop, step, ..
            } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    slice(list, *start, *stop, *step)
                        .into_iter()
                        .map(|(i, v)| Node::new_array_element(v, location, i as usize))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Selector::Wild { .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    list.iter()
                        .enumerate()
                        .map(|(i, v)| Node::new_array_element(v, location, i))
                        .collect()
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    dict.iter()
                        .map(|(k, v)| Node::new_object_member(v, location, k.extract().unwrap()))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Selector::Filter { expression, .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    list.iter()
                        .enumerate()
                        .map(|(i, v)| (i, v.clone(), expression.evaluate(&v, context)))
                        .filter(|(_, _, r)| is_truthy(r))
                        .map(|(i, v, _)| Node::new_array_element(v.clone(), location, i))
                        .collect()
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    dict.iter()
                        .map(|(k, v)| (k, v.clone(), expression.evaluate(&v, context)))
                        .filter(|(_, _, r)| is_truthy(r))
                        .map(|(k, v, _)| {
                            Node::new_object_member(v.clone(), location, k.extract().unwrap())
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

fn norm_index(index: i64, length: usize) -> usize {
    if index < 0 && length >= index.abs() as usize {
        (length as i64 + index) as usize
    } else {
        index as usize
    }
}

fn slice<'py>(
    list: &Bound<'py, PyList>,
    start: Option<i64>,
    stop: Option<i64>,
    step: Option<i64>,
) -> Vec<(i64, Bound<'py, PyAny>)> {
    let array_length = list.len() as i64; // TODO: try_from
    if array_length == 0 {
        return Vec::new();
    }

    let n_step = step.unwrap_or(1);

    if n_step == 0 {
        return Vec::new();
    }

    let n_start = match start {
        Some(i) => {
            if i < 0 {
                cmp::max(array_length + i, 0)
            } else {
                cmp::min(i, array_length - 1)
            }
        }
        None => {
            if n_step < 0 {
                array_length - 1
            } else {
                0
            }
        }
    };

    let n_stop = match stop {
        Some(i) => {
            if i < 0 {
                cmp::max(array_length + i, -1)
            } else {
                cmp::min(i, array_length)
            }
        }
        None => {
            if n_step < 0 {
                -1
            } else {
                array_length
            }
        }
    };

    let mut sliced_array: Vec<(i64, Bound<'py, PyAny>)> = Vec::new();

    // TODO: try_from instead of as
    if n_step > 0 {
        for i in (n_start..n_stop).step_by(n_step as usize) {
            sliced_array.push((i, list.get_item(i as usize).unwrap()));
        }
    } else {
        let mut i = n_start;
        while i > n_stop {
            sliced_array.push((i, list.get_item(i as usize).unwrap()));
            i += n_step;
        }
    }

    sliced_array
}

#[pymethods]
impl Selector {
    fn __repr__(&self) -> String {
        match self {
            Selector::Name { .. } => format!("<jpq.Selector.Name `{}`>", self),
            Selector::Index { .. } => format!("<jpq.Selector.Index `{}`>", self),
            Selector::Slice { .. } => format!("<jpq.Selector.Slice `{}`>", self),
            Selector::Wild { .. } => format!("<jpq.Selector.Wild `{}`>", self),
            Selector::Filter { .. } => format!("<jpq.Selector.Filter `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selector::Name { name, .. } => write!(f, "'{name}'"),
            Selector::Index {
                index: array_index, ..
            } => write!(f, "{array_index}"),
            Selector::Slice {
                start, stop, step, ..
            } => {
                write!(
                    f,
                    "{}:{}:{}",
                    start
                        .and_then(|i| Some(i.to_string()))
                        .unwrap_or(String::from("")),
                    stop.and_then(|i| Some(i.to_string()))
                        .unwrap_or(String::from("")),
                    step.and_then(|i| Some(i.to_string()))
                        .unwrap_or(String::from("1")),
                )
            }
            Selector::Wild { .. } => f.write_char('*'),
            Selector::Filter { expression, .. } => write!(f, "?{expression}"),
        }
    }
}
