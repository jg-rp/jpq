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
        span: (usize, usize),
        name: String,
    },
    Index {
        span: (usize, usize),
        index: i64,
    },
    Slice {
        span: (usize, usize),
        start: Option<i64>,
        stop: Option<i64>,
        step: Option<i64>,
    },
    Wild {
        span: (usize, usize),
    },
    Filter {
        span: (usize, usize),
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
                    nodes.push((val, format!("{}['{}']", node.1, name)));
                }
            }
            Selector::Index { index, .. } => {
                // We don't want to index Python strings or dicts with integer keys
                if value.is_instance_of::<PyList>() {
                    if let Ok(val) = value.get_item(index) {
                        if *index < 0 {
                            nodes.push((
                                val,
                                format!("{}[{}]", node.1, value.len().unwrap() as i64 + index), // TODO: try_from
                            ));
                        } else {
                            nodes.push((val, format!("{}[{}]", node.1, index)));
                        }
                    }
                }
            }
            Selector::Slice {
                start, stop, step, ..
            } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    for (i, element) in slice(list, *start, *stop, *step) {
                        nodes.push((element, format!("{}[{}]", node.1, i)))
                    }
                }
            }
            Selector::Wild { .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    for (i, element) in list.iter().enumerate() {
                        nodes.push((element, format!("{}[{}]", node.1, i)));
                    }
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, val) in dict.iter() {
                        nodes.push((val, format!("{}['{}']", node.1, key)));
                    }
                }
            }
            Selector::Filter { expression, .. } => {
                if let Ok(list) = value.downcast::<PyList>() {
                    for (i, element) in list.iter().enumerate() {
                        let filter_context = FilterContext {
                            env: context.env,
                            root: context.root.clone(),
                            current: element.clone(),
                        };
                        if is_truthy(&expression.evaluate(&filter_context)?) {
                            nodes.push((element, format!("{}[{}]", node.1, i)));
                        }
                    }
                } else if let Ok(dict) = value.downcast::<PyDict>() {
                    for (key, val) in dict.iter() {
                        let filter_context = FilterContext {
                            env: context.env,
                            root: context.root.clone(),
                            current: val.clone(),
                        };
                        if is_truthy(&expression.evaluate(&filter_context)?) {
                            nodes.push((val, format!("{}['{}']", node.1, key)));
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
