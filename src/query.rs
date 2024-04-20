//! Structs and enums that make up a JSONPath query syntax tree.
//!
//! The types in this module are used by the [`Parser`] to build an abstract
//! syntax tree for a JSONPath query. We are careful to use terminology from
//! [RFC 9535] and we model JSONPath segments and selectors explicitly.
//!
//! A [`Query`] contains zero or more [`Segment`]s, and each segment contains one
//! or more [`Selector`]s. When a segment includes a _filter selector_, that
//! filter selector is a tree of [`FilterExpression`]s.
//!
//! [RFC 9535]: https://datatracker.ietf.org/doc/html/rfc9535

use lazy_static::lazy_static;
use pyo3::prelude::*;
use std::fmt::{self, Write};

use crate::{errors::JSONPathError, parser::Parser};

lazy_static! {
    static ref PARSER: Parser = Parser::new();
}

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

impl Query {
    pub fn new(segments: Vec<Segment>) -> Self {
        Query { segments }
    }

    pub fn standard(expr: &str) -> Result<Self, JSONPathError> {
        PARSER.parse(expr)
    }

    pub fn is_empty(&self) -> bool {
        self.segments.len() == 0
    }

    pub fn is_singular(&self) -> bool {
        for segment in self.segments.iter() {
            match segment {
                Segment::Child { selectors, .. } => {
                    // A single name or index selector?
                    if selectors.len() == 1
                        && selectors.first().is_some_and(|s| {
                            matches!(s, Selector::Name { .. } | Selector::Index { .. })
                        })
                    {
                        continue;
                    } else {
                        return false;
                    }
                }
                Segment::Recursive { .. } => {
                    return false;
                }
            }
        }

        true
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
    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("selectors", "span");

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

impl<'py> pyo3::FromPyObject<'py> for Box<FilterExpression> {
    fn extract(ob: &'py PyAny) -> PyResult<Self> {
        ob.extract::<FilterExpression>().map(Box::new)
    }
}

impl pyo3::IntoPy<pyo3::PyObject> for Box<FilterExpression> {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        (*self).into_py(py)
    }
}

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

#[pyclass]
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::And => f.write_str("&&"),
            LogicalOperator::Or => f.write_str("||"),
        }
    }
}

#[pymethods]
impl LogicalOperator {
    fn __repr__(&self) -> String {
        match self {
            LogicalOperator::And => format!("<jpq.LogicalOperator.And `{}`>", self),
            LogicalOperator::Or => format!("<jpq.LogicalOperator.Or `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::Eq => f.write_str("=="),
            ComparisonOperator::Ne => f.write_str("!="),
            ComparisonOperator::Ge => f.write_str(">="),
            ComparisonOperator::Gt => f.write_str(">"),
            ComparisonOperator::Le => f.write_str("<="),
            ComparisonOperator::Lt => f.write_str("<"),
        }
    }
}

#[pymethods]
impl ComparisonOperator {
    fn __repr__(&self) -> String {
        match self {
            ComparisonOperator::Eq => format!("<jpq.ComparisonOperator.Eq `{}`>", self),
            ComparisonOperator::Ne => format!("<jpq.ComparisonOperator.Ne `{}`>", self),
            ComparisonOperator::Ge => format!("<jpq.ComparisonOperator.Ge `{}`>", self),
            ComparisonOperator::Gt => format!("<jpq.ComparisonOperator.Gt `{}`>", self),
            ComparisonOperator::Le => format!("<jpq.ComparisonOperator.Ler `{}`>", self),
            ComparisonOperator::Lt => format!("<jpq.ComparisonOperator.Ltr `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum FilterExpression {
    True_ {
        span: (usize, usize),
    },
    False_ {
        span: (usize, usize),
    },
    Null {
        span: (usize, usize),
    },
    String {
        span: (usize, usize),
        value: String,
    },
    Int {
        span: (usize, usize),
        value: i64,
    },
    Float {
        span: (usize, usize),
        value: f64,
    },
    Not {
        span: (usize, usize),
        expression: Box<FilterExpression>,
    },
    Logical {
        span: (usize, usize),
        left: Box<FilterExpression>,
        operator: LogicalOperator,
        right: Box<FilterExpression>,
    },
    Comparison {
        span: (usize, usize),
        left: Box<FilterExpression>,
        operator: ComparisonOperator,
        right: Box<FilterExpression>,
    },
    RelativeQuery {
        span: (usize, usize),
        query: Box<Query>,
    },
    RootQuery {
        span: (usize, usize),
        query: Box<Query>,
    },
    Function {
        span: (usize, usize),
        name: String,
        args: Vec<FilterExpression>,
    },
}

impl FilterExpression {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            FilterExpression::True_ { .. }
                | FilterExpression::False_ { .. }
                | FilterExpression::Null { .. }
                | FilterExpression::String { .. }
                | FilterExpression::Int { .. }
                | FilterExpression::Float { .. }
        )
    }

    pub fn span(&self) -> (usize, usize) {
        match self {
            FilterExpression::True_ { span, .. }
            | FilterExpression::False_ { span, .. }
            | FilterExpression::Null { span, .. }
            | FilterExpression::String { span, .. }
            | FilterExpression::Int { span, .. }
            | FilterExpression::Float { span, .. }
            | FilterExpression::Not { span, .. }
            | FilterExpression::Logical { span, .. }
            | FilterExpression::Comparison { span, .. }
            | FilterExpression::RelativeQuery { span, .. }
            | FilterExpression::RootQuery { span, .. }
            | FilterExpression::Function { span, .. } => *span,
        }
    }
}

impl fmt::Display for FilterExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterExpression::True_ { .. } => f.write_str("true"),
            FilterExpression::False_ { .. } => f.write_str("false"),
            FilterExpression::Null { .. } => f.write_str("null"),
            FilterExpression::String { value, .. } => write!(f, "\"{value}\""),
            FilterExpression::Int { value, .. } => write!(f, "{value}"),
            FilterExpression::Float { value, .. } => write!(f, "{value}"),
            FilterExpression::Not { expression, .. } => write!(f, "!{expression}"),
            FilterExpression::Logical {
                left,
                operator,
                right,
                ..
            } => write!(f, "({left} {operator} {right})"),
            FilterExpression::Comparison {
                left,
                operator,
                right,
                ..
            } => write!(f, "{left} {operator} {right}"),
            FilterExpression::RelativeQuery { query, .. } => {
                write!(
                    f,
                    "@{}",
                    query
                        .segments
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
            FilterExpression::RootQuery { query, .. } => {
                write!(
                    f,
                    "${}",
                    query
                        .segments
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
            FilterExpression::Function { name, args, .. } => {
                write!(
                    f,
                    "{}({})",
                    name,
                    args.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

#[pymethods]
impl FilterExpression {
    fn __repr__(&self) -> String {
        match self {
            FilterExpression::True_ { .. } => "<jpq.FilterExpression.True>".to_string(),
            FilterExpression::False_ { .. } => "<jpq.FilterExpression.False>".to_string(),
            FilterExpression::Null { .. } => "<jpq.FilterExpression.Null>".to_string(),
            FilterExpression::String { .. } => {
                format!("<jpq.FilterExpression.String `{}`>", self)
            }
            FilterExpression::Int { .. } => format!("<jpq.FilterExpression.Int {}>", self),
            FilterExpression::Float { .. } => {
                format!("<jpq.FilterExpression.Float `{}`>", self)
            }
            FilterExpression::Not { .. } => {
                format!("<jpq.FilterExpression.Not `{}`>", self)
            }
            FilterExpression::Logical { .. } => {
                format!("<jpq.FilterExpression.Logical `{}`>", self)
            }
            FilterExpression::Comparison { .. } => {
                format!("<jpq.FilterExpression.Comparison `{}`>", self)
            }
            FilterExpression::RelativeQuery { .. } => {
                format!("<jpq.FilterExpression.RelativeQuery `{}`>", self)
            }
            FilterExpression::RootQuery { .. } => {
                format!("<jpq.FilterExpression.RootQuery `{}`>", self)
            }
            FilterExpression::Function { .. } => {
                format!("<jpq.FilterExpression.Function `{}`>", self)
            }
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}
