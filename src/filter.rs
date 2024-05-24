use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyNone, PyString, PyTuple};

use crate::{ExpressionType, FilterContext, JSONPathError, NodeList, Query};

#[pyclass]
#[derive(Debug, Clone)]
pub enum FilterExpression {
    True_ {},
    False_ {},
    Null {},
    StringLiteral {
        value: String,
    },
    Int {
        value: i64,
    },
    Float {
        value: f64,
    },
    Not {
        expression: Box<FilterExpression>,
    },
    Logical {
        left: Box<FilterExpression>,
        operator: LogicalOperator,
        right: Box<FilterExpression>,
    },
    Comparison {
        left: Box<FilterExpression>,
        operator: ComparisonOperator,
        right: Box<FilterExpression>,
    },
    RelativeQuery {
        query: Box<Query>,
    },
    RootQuery {
        query: Box<Query>,
    },
    Function {
        name: String,
        args: Vec<FilterExpression>,
    },
    CurrentKey {},
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
            LogicalOperator::And => format!("<jpq.LogicalOp.And `{}`>", self),
            LogicalOperator::Or => format!("<jpq.LogicalOp.Or `{}`>", self),
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
            ComparisonOperator::Eq => format!("<jpq.ComparisonOp.Eq `{}`>", self),
            ComparisonOperator::Ne => format!("<jpq.ComparisonOp.Ne `{}`>", self),
            ComparisonOperator::Ge => format!("<jpq.ComparisonOp.Ge `{}`>", self),
            ComparisonOperator::Gt => format!("<jpq.ComparisonOp.Gt `{}`>", self),
            ComparisonOperator::Le => format!("<jpq.ComparisonOp.Ler `{}`>", self),
            ComparisonOperator::Lt => format!("<jpq.ComparisonOp.Ltr `{}`>", self),
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
            Function { name, args } => {
                let obj = context
                    .env
                    .function_register
                    .bind(py)
                    .get_item(name)
                    .map_err(|_| {
                        JSONPathError::name(format!("missing function definition for {}", name))
                    })?
                    .ok_or_else(|| {
                        JSONPathError::name(format!("missing function definition for {}", name))
                    })?;

                let sig = context
                    .env
                    .parser
                    .function_signatures
                    .get(name)
                    .ok_or_else(|| {
                        JSONPathError::name(format!("missing function signature for {}", name))
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
                    .map_err(|err| JSONPathError::ext(err.to_string()))?;

                match sig.return_type {
                    ExpressionType::Nodes => Ok(Nodes(
                        rv.extract()
                            .map_err(|err| JSONPathError::ext(err.to_string()))?,
                    )),
                    _ => Ok(Object(rv)),
                }
            }
            CurrentKey { .. } => {
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
            CurrentKey { .. } => f.write_str("#"),
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
            CurrentKey { .. } => "<jpq.FilterExpression.Key>".to_string(),
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
