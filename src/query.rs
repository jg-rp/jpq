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
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString, PyTuple};

use std::cmp;
use std::fmt::{self, Write};

use crate::environment::Env;
use crate::ExpressionType;
use crate::{errors::JSONPathError, parser::Parser};

lazy_static! {
    static ref PARSER: Parser = Parser::new();
}

pub type Node<'py> = (Bound<'py, PyAny>, String);
pub type NodeList<'py> = Vec<Node<'py>>;

pub struct QueryContext<'py> {
    env: &'py Env,
    root: Bound<'py, PyAny>,
}

pub struct FilterContext<'py> {
    env: &'py Env,
    root: Bound<'py, PyAny>,
    current: Bound<'py, PyAny>,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Query {
    #[pyo3(get)]
    pub segments: Vec<Segment>,
    // TODO: env
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

    pub fn resolve<'py>(&self, value: &Bound<'py, PyAny>, env: &Env) -> NodeList<'py> {
        let context = QueryContext {
            env,
            root: value.clone(),
        };

        let mut nodes: NodeList<'py> = vec![(value.clone(), "$".to_owned())];

        // TODO: try iter().scan()
        for segment in self.segments.iter() {
            nodes = segment.resolve(nodes, &context);
        }

        nodes
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

    // TODO: fn find
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

impl Segment {
    pub fn resolve<'py>(&self, nodes: NodeList<'py>, context: &QueryContext) -> NodeList<'py> {
        match self {
            Segment::Child { selectors, .. } => {
                let mut _nodes: NodeList = Vec::new();
                for node in nodes.iter() {
                    for selector in selectors {
                        _nodes.extend(selector.resolve(node, context));
                    }
                }
                _nodes
            }
            Segment::Recursive { selectors, .. } => {
                let mut _nodes: NodeList = Vec::new();
                for node in nodes {
                    for _node in self.visit(node).iter() {
                        for selector in selectors {
                            _nodes.extend(selector.resolve(_node, context));
                        }
                    }
                }
                _nodes
            }
        }
    }

    fn visit<'py>(&self, node: Node<'py>) -> NodeList<'py> {
        // TODO: less cloning
        let mut nodes: NodeList = vec![node.clone()];
        let value = &node.0;
        let loc = &node.1;

        if let Ok(list) = value.downcast::<PyList>() {
            for (i, element) in list.iter().enumerate() {
                let _node = (element, format!("{}[{}]", loc, i));
                let children = self.visit(_node.clone());
                // nodes.push(_node);
                nodes.extend(children)
            }
        } else if let Ok(dict) = value.downcast::<PyDict>() {
            for (key, val) in dict.iter() {
                let _node = (val, format!("{}['{}']", loc, key));
                let children = self.visit(_node.clone());
                // nodes.push(_node);
                nodes.extend(children);
            }
        }

        nodes
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

impl Selector {
    fn resolve<'py>(&self, node: &Node<'py>, context: &QueryContext) -> NodeList<'py> {
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
                    // TODO: normalize index
                    if let Ok(val) = value.get_item(index) {
                        nodes.push((val, format!("{}[{}]", node.1, index)));
                    }
                }
            }
            Selector::Slice {
                start, stop, step, ..
            } => {
                // TODO: make sure we don't truncate indexes on 32-bit systems
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
                        if is_truthy(&expression.evaluate(&filter_context)) {
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
                        if is_truthy(&expression.evaluate(&filter_context)) {
                            nodes.push((val, format!("{}['{}']", node.1, key)));
                        }
                    }
                }
            }
        }
        nodes
    }
}

fn slice<'py>(
    list: &Bound<'py, PyList>,
    start: Option<i64>,
    stop: Option<i64>,
    step: Option<i64>,
) -> Vec<(i64, Bound<'py, PyAny>)> {
    let array_length = list.len() as i64;
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

pub enum FilterExpressionResult<'py> {
    Object(Bound<'py, PyAny>),
    Nodes(NodeList<'py>),
    Nothing,
}

macro_rules! any_bool {
    ($py:expr, $value:expr) => {
        FilterExpressionResult::Object(PyBool::new_bound($py, $value).as_any().to_owned())
    };
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

    pub fn evaluate<'py>(&self, context: &'py FilterContext) -> FilterExpressionResult<'py> {
        use FilterExpression::*;
        use FilterExpressionResult::*;
        let py = context.root.py();
        match self {
            True_ { .. } => any_bool!(py, true),
            False_ { .. } => any_bool!(py, false),
            Null { .. } => Object(PyNone::get_bound(py).as_any().to_owned()),
            String { value, .. } => Object(PyString::new_bound(py, value).as_any().to_owned()),
            Int { value, .. } => Object(value.to_object(py).bind(py).to_owned()),
            Float { value, .. } => Object(value.to_object(py).bind(py).to_owned()),
            Not { expression, .. } => {
                any_bool!(py, !is_truthy(&expression.evaluate(context)))
            }
            Logical {
                left,
                operator,
                right,
                ..
            } => {
                any_bool!(
                    py,
                    logical(&left.evaluate(context), operator, &right.evaluate(context),)
                )
            }
            Comparison {
                left,
                operator,
                right,
                ..
            } => {
                let mut _left = left.evaluate(context);
                if let Nodes(nodes) = &_left {
                    if nodes.len() == 1 {
                        _left = Object(nodes.first().unwrap().0.clone());
                    }
                }

                let mut _right = right.evaluate(context);
                if let Nodes(nodes) = &_right {
                    if nodes.len() == 1 {
                        _right = Object(nodes.first().unwrap().0.clone());
                    }
                }

                any_bool!(py, compare(&_left, operator, &_right))
            }
            RelativeQuery { query, .. } => Nodes(query.resolve(&context.current, context.env)),
            RootQuery { query, .. } => Nodes(query.resolve(&context.root, context.env)),
            Function { name, args, .. } => {
                if let Ok(Some(obj)) = context.env.function_register.bind(py).get_item(name) {
                    // TODO: error if obj not callable

                    // TODO: look up function signature here, once

                    let _args = args
                        .iter()
                        .map(|ex| ex.evaluate(context))
                        .enumerate()
                        .map(|(i, rv)| unpack_result(name, rv, i, context.env, py))
                        .collect::<Vec<Bound<'py, PyAny>>>();

                    // TODO: handle errors
                    let rv = obj.call1(PyTuple::new_bound(py, _args)).unwrap();

                    let return_type = context
                        .env
                        .parser
                        .function_types
                        .get(name)
                        .map(|sig| sig.return_type)
                        .unwrap(); // TODO: return error

                    if matches!(return_type, ExpressionType::Nodes) {
                        Nodes(rv.extract().unwrap()) // TODO: error
                    } else {
                        Object(rv)
                    }
                } else {
                    // TODO: error: function not found
                    todo!()
                }
            }
        }
    }
}

fn unpack_result<'py>(
    func_name: &str,
    rv: FilterExpressionResult<'py>,
    index: usize,
    env: &'py Env,
    py: Python<'py>,
) -> Bound<'py, PyAny> {
    let arg_type = env
        .parser
        .function_types
        .get(func_name)
        .and_then(|sig| sig.param_types.get(index))
        .unwrap(); // TODO: return error

    match rv {
        FilterExpressionResult::Nodes(nodes) => {
            if !matches!(arg_type, ExpressionType::Nodes) {
                match nodes.len() {
                    0 => env.nothing.bind(py).clone(),
                    1 => nodes.get(0).unwrap().0.clone(),
                    _ => {
                        let object = &nodes.to_object(py);
                        object.bind(py).clone()
                    }
                }
            } else {
                let object = &nodes.to_object(py);
                object.bind(py).clone()
            }
        }
        FilterExpressionResult::Nothing => env.nothing.bind(py).clone(),
        FilterExpressionResult::Object(obj) => obj,
    }
}

impl fmt::Display for FilterExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: use
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
        // TODO: use
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

fn is_truthy(rv: &FilterExpressionResult) -> bool {
    use FilterExpressionResult::*;
    match rv {
        Nothing => false,
        Nodes(nodes) => nodes.len() > 0,
        Object(obj) => obj.is_truthy().unwrap(),
    }
}

fn logical(
    left: &FilterExpressionResult,
    op: &LogicalOperator,
    right: &FilterExpressionResult,
) -> bool {
    match op {
        LogicalOperator::And => is_truthy(left) && is_truthy(right),
        LogicalOperator::Or => is_truthy(left) || is_truthy(right),
    }
}

fn compare(
    left: &FilterExpressionResult,
    op: &ComparisonOperator,
    right: &FilterExpressionResult,
) -> bool {
    use ComparisonOperator::*;
    match op {
        Eq => eq((left, right)),
        Ne => !eq((left, right)),
        Lt => lt((left, right)),
        Gt => lt((right, left)),
        Ge => lt((right, left)) || eq((left, right)),
        Le => lt((left, right)) || eq((left, right)),
    }
}

fn eq(pair: (&FilterExpressionResult, &FilterExpressionResult)) -> bool {
    use FilterExpressionResult::*;
    match pair {
        (Nodes(left), Nodes(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(l, r)| l.0.eq(&r.0).unwrap())
        }
        (Nodes(nodes), Nothing) | (Nothing, Nodes(nodes)) => nodes.is_empty(),
        (Nodes(nodes), Object(obj)) | (Object(obj), Nodes(nodes)) => {
            if nodes.len() == 1 {
                obj.eq(&nodes.get(0).unwrap().0).unwrap()
            } else {
                false
            }
        }
        (Nothing, Nothing) => true,
        (Nothing, Object(..)) | (Object(..), Nothing) => false,
        (Object(left), Object(right)) => left.eq(right).unwrap(),
    }
}

fn lt(pair: (&FilterExpressionResult, &FilterExpressionResult)) -> bool {
    use FilterExpressionResult::*;
    match pair {
        (Object(left), Object(right)) => {
            if left.is_instance_of::<PyString>() && right.is_instance_of::<PyString>() {
                left.lt(right).unwrap()
            } else if left.is_instance_of::<PyBool>() || right.is_instance_of::<PyBool>() {
                false
            } else if (left.is_instance_of::<PyInt>() || left.is_instance_of::<PyFloat>())
                && (right.is_instance_of::<PyInt>() || right.is_instance_of::<PyFloat>())
            {
                left.lt(right).unwrap()
            } else {
                false
            }
        }
        _ => false,
    }
}
