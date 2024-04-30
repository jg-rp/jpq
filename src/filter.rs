use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyNone, PyString, PyTuple};

use crate::{ExpressionType, FilterContext, JSONPathError, NodeList, Query};

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
    StringLiteral {
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
        operator: LogicalOp,
        right: Box<FilterExpression>,
    },
    Comparison {
        span: (usize, usize),
        left: Box<FilterExpression>,
        operator: ComparisonOp,
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
    Key {
        span: (usize, usize),
    },
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
pub enum LogicalOp {
    And,
    Or,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => f.write_str("&&"),
            LogicalOp::Or => f.write_str("||"),
        }
    }
}

#[pymethods]
impl LogicalOp {
    fn __repr__(&self) -> String {
        match self {
            LogicalOp::And => format!("<jpq.LogicalOp.And `{}`>", self),
            LogicalOp::Or => format!("<jpq.LogicalOp.Or `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::Eq => f.write_str("=="),
            ComparisonOp::Ne => f.write_str("!="),
            ComparisonOp::Ge => f.write_str(">="),
            ComparisonOp::Gt => f.write_str(">"),
            ComparisonOp::Le => f.write_str("<="),
            ComparisonOp::Lt => f.write_str("<"),
        }
    }
}

#[pymethods]
impl ComparisonOp {
    fn __repr__(&self) -> String {
        match self {
            ComparisonOp::Eq => format!("<jpq.ComparisonOp.Eq `{}`>", self),
            ComparisonOp::Ne => format!("<jpq.ComparisonOp.Ne `{}`>", self),
            ComparisonOp::Ge => format!("<jpq.ComparisonOp.Ge `{}`>", self),
            ComparisonOp::Gt => format!("<jpq.ComparisonOp.Gt `{}`>", self),
            ComparisonOp::Le => format!("<jpq.ComparisonOp.Ler `{}`>", self),
            ComparisonOp::Lt => format!("<jpq.ComparisonOp.Ltr `{}`>", self),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
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
                | FilterExpression::StringLiteral { .. }
                | FilterExpression::Int { .. }
                | FilterExpression::Float { .. }
        )
    }

    pub fn span(&self) -> (usize, usize) {
        match self {
            FilterExpression::True_ { span, .. }
            | FilterExpression::False_ { span, .. }
            | FilterExpression::Null { span, .. }
            | FilterExpression::StringLiteral { span, .. }
            | FilterExpression::Int { span, .. }
            | FilterExpression::Float { span, .. }
            | FilterExpression::Not { span, .. }
            | FilterExpression::Logical { span, .. }
            | FilterExpression::Comparison { span, .. }
            | FilterExpression::RelativeQuery { span, .. }
            | FilterExpression::RootQuery { span, .. }
            | FilterExpression::Function { span, .. }
            | FilterExpression::Key { span } => *span,
        }
    }

    pub fn evaluate<'py>(
        &self,
        context: &'py FilterContext,
    ) -> Result<FilterExpressionResult<'py>, JSONPathError> {
        use FilterExpression::*;
        use FilterExpressionResult::*;
        let py = context.root.py();
        match self {
            True_ { .. } => Ok(any_bool!(py, true)),
            False_ { .. } => Ok(any_bool!(py, false)),
            Null { .. } => Ok(Object(PyNone::get_bound(py).as_any().to_owned())),
            FilterExpression::StringLiteral { value, .. } => {
                Ok(Object(PyString::new_bound(py, value).as_any().to_owned()))
            }
            Int { value, .. } => Ok(Object(value.to_object(py).bind(py).to_owned())),
            Float { value, .. } => Ok(Object(value.to_object(py).bind(py).to_owned())),
            Not { expression, .. } => Ok(any_bool!(py, !is_truthy(&expression.evaluate(context)?))),
            Logical {
                left,
                operator,
                right,
                ..
            } => Ok(any_bool!(
                py,
                logical(
                    &left.evaluate(context)?,
                    operator,
                    &right.evaluate(context)?,
                )
            )),
            Comparison {
                left,
                operator,
                right,
                ..
            } => {
                let mut _left = left.evaluate(context)?;
                if let Nodes(nodes) = &_left {
                    if nodes.len() == 1 {
                        _left = Object(nodes.first().unwrap().0.clone());
                    }
                }

                let mut _right = right.evaluate(context)?;
                if let Nodes(nodes) = &_right {
                    if nodes.len() == 1 {
                        _right = Object(nodes.first().unwrap().0.clone());
                    }
                }

                Ok(any_bool!(py, compare(&_left, operator, &_right)))
            }
            RelativeQuery { query, .. } => Ok(Nodes(query.resolve(&context.current, context.env)?)),
            RootQuery { query, .. } => Ok(Nodes(query.resolve(&context.root, context.env)?)),
            Function { name, args, span } => {
                let obj = context
                    .env
                    .function_register
                    .bind(py)
                    .get_item(name)
                    .map_err(|_| {
                        JSONPathError::name(
                            format!("missing function definition for {}", name),
                            *span,
                        )
                    })?
                    .ok_or_else(|| {
                        JSONPathError::name(
                            format!("missing function definition for {}", name),
                            *span,
                        )
                    })?;

                let sig = context.env.parser.function_types.get(name).ok_or_else(|| {
                    JSONPathError::name(format!("missing function signature for {}", name), *span)
                })?;

                let _args: Result<Vec<_>, _> = args
                    .iter()
                    .map(|ex| ex.evaluate(context))
                    .enumerate()
                    .map(|(i, rv)| {
                        unpack_result(
                            rv?,
                            &sig.param_types,
                            i,
                            context.env.nothing.clone().bind(py),
                            py,
                        )
                    })
                    .collect();

                let rv = obj
                    .call1(PyTuple::new_bound(py, _args?))
                    .map_err(|err| JSONPathError::ext(err.to_string(), *span))?;

                match sig.return_type {
                    ExpressionType::Nodes => {
                        Ok(Nodes(rv.extract().map_err(|err| {
                            JSONPathError::ext(err.to_string(), *span)
                        })?))
                    }
                    _ => Ok(Object(rv)),
                }
            }
            Key { .. } => {
                if let Some(key) = &context.current_key {
                    Ok(Object(key.clone()))
                } else {
                    Ok(Nothing)
                }
            }
        }
    }
}

fn unpack_result<'py>(
    rv: FilterExpressionResult<'py>,
    param_types: &[ExpressionType],
    index: usize,
    nothing: &Bound<'py, PyAny>,
    py: Python<'py>,
) -> Result<Bound<'py, PyAny>, JSONPathError> {
    let arg_type = param_types.get(index).unwrap();

    match rv {
        FilterExpressionResult::Nodes(nodes) => {
            if !matches!(arg_type, ExpressionType::Nodes) {
                match nodes.len() {
                    0 => Ok(nothing.clone()),
                    1 => Ok(nodes.first().unwrap().0.clone()),
                    _ => {
                        let object = &nodes.to_object(py);
                        Ok(object.bind(py).clone())
                    }
                }
            } else {
                let object = &nodes.to_object(py);
                Ok(object.bind(py).clone())
            }
        }
        FilterExpressionResult::Nothing => Ok(nothing.clone()),
        FilterExpressionResult::Object(obj) => Ok(obj),
    }
}

impl fmt::Display for FilterExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FilterExpression::*;
        match self {
            True_ { .. } => f.write_str("true"),
            False_ { .. } => f.write_str("false"),
            Null { .. } => f.write_str("null"),
            StringLiteral { value, .. } => write!(f, "\"{value}\""),
            Int { value, .. } => write!(f, "{value}"),
            Float { value, .. } => write!(f, "{value}"),
            Not { expression, .. } => write!(f, "!{expression}"),
            Logical {
                left,
                operator,
                right,
                ..
            } => write!(f, "({left} {operator} {right})"),
            Comparison {
                left,
                operator,
                right,
                ..
            } => write!(f, "{left} {operator} {right}"),
            RelativeQuery { query, .. } => {
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
            RootQuery { query, .. } => {
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
            Function { name, args, .. } => {
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
            Key { .. } => f.write_str("#"),
        }
    }
}

#[pymethods]
impl FilterExpression {
    fn __repr__(&self) -> String {
        use FilterExpression::*;
        match self {
            True_ { .. } => "<jpq.FilterExpression.True>".to_string(),
            False_ { .. } => "<jpq.FilterExpression.False>".to_string(),
            Null { .. } => "<jpq.FilterExpression.Null>".to_string(),
            StringLiteral { .. } => {
                format!("<jpq.FilterExpression.String `{}`>", self)
            }
            Int { .. } => format!("<jpq.FilterExpression.Int {}>", self),
            Float { .. } => {
                format!("<jpq.FilterExpression.Float `{}`>", self)
            }
            Not { .. } => {
                format!("<jpq.FilterExpression.Not `{}`>", self)
            }
            Logical { .. } => {
                format!("<jpq.FilterExpression.Logical `{}`>", self)
            }
            Comparison { .. } => {
                format!("<jpq.FilterExpression.Comparison `{}`>", self)
            }
            RelativeQuery { .. } => {
                format!("<jpq.FilterExpression.RelativeQuery `{}`>", self)
            }
            RootQuery { .. } => {
                format!("<jpq.FilterExpression.RootQuery `{}`>", self)
            }
            Function { .. } => {
                format!("<jpq.FilterExpression.Function `{}`>", self)
            }
            Key { .. } => "<jpq.FilterExpression.Key>".to_string(),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

pub fn is_truthy(rv: &FilterExpressionResult) -> bool {
    use FilterExpressionResult::*;
    match rv {
        Nothing => false,
        Nodes(nodes) => !nodes.is_empty(),
        Object(obj) => obj.is_truthy().unwrap(),
    }
}

fn logical(left: &FilterExpressionResult, op: &LogicalOp, right: &FilterExpressionResult) -> bool {
    match op {
        LogicalOp::And => is_truthy(left) && is_truthy(right),
        LogicalOp::Or => is_truthy(left) || is_truthy(right),
    }
}

fn compare(
    left: &FilterExpressionResult,
    op: &ComparisonOp,
    right: &FilterExpressionResult,
) -> bool {
    use ComparisonOp::*;
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
                obj.eq(&nodes.first().unwrap().0).unwrap()
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
