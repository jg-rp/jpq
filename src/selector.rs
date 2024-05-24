use std::cmp;
use std::fmt::{self, Write};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::filter::{is_truthy, FilterExpression};
use crate::{FilterContext, JSONPathError, Node, NodeList, QueryContext};

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
    Key {
        name: String,
    },
    Keys {},
    KeysFilter {
        expression: Box<FilterExpression>,
    },
}

impl Selector {
    pub fn resolve<'py>(
        &self,
        node: &Node<'py>,
        context: &QueryContext,
    ) -> Result<NodeList<'py>, JSONPathError> {
        let mut nodes: NodeList = Vec::new();
        let value = &node.0;
        match self {
            Selector::Name { name, .. } => {
                if let Ok(val) = value.get_item(name) {
                    nodes.push((
                        val,
                        format!("{}['{}']", node.1, name),
                        name.into_py(value.py()),
                    ));
                }
            }
            Selector::Index { index, .. } => {
                // We don't want to index Python strings or dicts with integer keys
                if value.is_instance_of::<PyList>() {
                    if let Ok(val) = value.get_item(index) {
                        if *index < 0 {
                            let i = value.len().unwrap() as i64 + index; // TODO: try_from
                            nodes.push((val, format!("{}[{}]", node.1, i), i.into_py(value.py())));
                        } else {
                            nodes.push((
                                val,
                                format!("{}[{}]", node.1, index),
                                index.into_py(value.py()),
                            ));
                        }
                    }
                }
            }
            Selector::Slice {
                start, stop, step, ..
            } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    let py = list.py();
                    for (i, element) in slice(list, *start, *stop, *step) {
                        nodes.push((element, format!("{}[{}]", node.1, i), i.into_py(py)))
                    }
                }
            }
            Selector::Wild { .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    let py = list.py();
                    for (i, element) in list.iter().enumerate() {
                        nodes.push((element, format!("{}[{}]", node.1, i), i.into_py(py)));
                    }
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, val) in dict.iter() {
                        nodes.push((val, format!("{}['{}']", node.1, key), key.unbind()));
                    }
                }
            }
            Selector::Filter { expression, .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    let py = list.py();
                    for (i, element) in list.iter().enumerate() {
                        let py_i = i.to_object(node.0.py());
                        let filter_context = FilterContext {
                            env: context.env,
                            root: context.root.clone(),
                            current: element.clone(),
                            current_key: Some(py_i.bind(node.0.py()).clone()),
                        };
                        if is_truthy(&expression.evaluate(&filter_context)?) {
                            nodes.push((element, format!("{}[{}]", node.1, i), i.into_py(py)));
                        }
                    }
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, val) in dict.iter() {
                        let filter_context = FilterContext {
                            env: context.env,
                            root: context.root.clone(),
                            current: val.clone(),
                            current_key: Some(key.clone()),
                        };
                        if is_truthy(&expression.evaluate(&filter_context)?) {
                            nodes.push((val, format!("{}['{}']", node.1, key), key.unbind()));
                        }
                    }
                }
            }
            Selector::Key { name, .. } => {
                // Non-standard
                if let Ok(_val) = value.get_item(name) {
                    // TODO: escape `'`
                    let py_name = name.to_object(node.0.py());
                    nodes.push((
                        py_name.bind(node.0.py()).clone(),
                        format!("{}[~'{}']", node.1, name),
                        py_name,
                    ));
                }
            }
            Selector::Keys { .. } => {
                // Non-standard
                if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, _val) in dict.iter() {
                        // TODO: escape `'`
                        nodes.push((key.clone(), format!("{}[~'{}']", node.1, key), key.unbind()));
                    }
                }
            }
            Selector::KeysFilter { expression, .. } => {
                // Non-standard
                if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, val) in dict.iter() {
                        let filter_context = FilterContext {
                            env: context.env,
                            root: context.root.clone(),
                            current: val.clone(),
                            current_key: Some(key.clone()),
                        };
                        if is_truthy(&expression.evaluate(&filter_context)?) {
                            // TODO: escape `'`
                            nodes.push((
                                key.clone(),
                                format!("{}[~'{}']", node.1, key),
                                key.unbind(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(nodes)
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
            Selector::Key { .. } => format!("<jpq.Selector.Key `{}`>", self),
            Selector::Keys { .. } => format!("<jpq.Selector.Keys `{}`>", self),
            Selector::KeysFilter { .. } => format!("<jpq.Selector.KeysFilter `{}`>", self),
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
            Selector::Key { name, .. } => write!(f, "~'{name}'"), // TODO: escape `'`
            Selector::Keys { .. } => f.write_char('~'),
            Selector::KeysFilter { expression, .. } => write!(f, "~?{expression}"),
        }
    }
}
