from __future__ import annotations
from contextlib import suppress
from typing import TYPE_CHECKING
from typing import Iterable


from jpq.jpq import (
    ComparisonOperator,
    LogicalOperator,
    Query,
    Selector,
    Segment,
    FilterExpression,
)

from .node import JSONPathNode
from .node import JSONPathNodeList
from .node import JSONValue
from .nothing import NOTHING

from jpq.function_extensions.filter_function import ExpressionType
from jpq.function_extensions.filter_function import FilterFunction

if TYPE_CHECKING:
    from .env import JSONPathEnvironment
    from jpq.jpq import Selector_
    from jpq.jpq import FilterExpression_

# TODO: configure ruff
# TODO: configure mypy


class JSONPathQuery:
    __slots__ = ("_query", "env")

    def __init__(
        self,
        query: Query,
        *,
        env: JSONPathEnvironment,
    ) -> None:
        self._query = query
        self.env = env

    def __str__(self) -> str:
        return str(self._query)

    def finditer(self, value: JSONValue) -> Iterable[JSONPathNode]:
        """"""
        nodes: Iterable[JSONPathNode] = [
            JSONPathNode(
                value=value,
                location=(),
            )
        ]

        for segment in self._query.segments:
            match segment:
                case Segment.Child(selectors):
                    nodes = resolve_child_segment(
                        env=self.env,
                        root=value,
                        nodes=nodes,
                        selectors=selectors,
                    )
                case Segment.Recursive(selectors):
                    nodes = resolve_descendant_segment(
                        env=self.env,
                        root=value,
                        nodes=nodes,
                        selectors=selectors,
                    )
                case _:
                    # TODO: raise proper exception
                    raise ValueError(":(")

        return nodes

    def find(self, value: JSONValue) -> JSONPathNodeList:
        return JSONPathNodeList(self.finditer(value))


def resolve_child_segment(
    *,
    env: JSONPathEnvironment,
    root: JSONValue,
    nodes: Iterable[JSONPathNode],
    selectors: list[Selector_],
) -> Iterable[JSONPathNode]:
    """"""
    for node in (node for node in nodes if isinstance(node.value, (dict, list))):
        for selector in selectors:
            yield from resolve_selector(
                env=env,
                node=node,
                root=root,
                selector=selector,
            )


def resolve_descendant_segment(
    *,
    env: JSONPathEnvironment,
    root: JSONValue,
    nodes: Iterable[JSONPathNode],
    selectors: list[Selector_],
) -> Iterable[JSONPathNode]:
    """"""
    for node in (node for node in nodes if isinstance(node.value, (dict, list))):
        for _node in _visit(node):
            for selector in selectors:
                yield from resolve_selector(
                    env=env,
                    node=_node,
                    root=root,
                    selector=selector,
                )


def _visit(node: JSONPathNode, depth: int = 1) -> Iterable[JSONPathNode]:
    """Depth-first, pre-order node traversal."""
    # TODO: check recursion depth
    # if depth > self.env.max_recursion_depth:
    #     raise JSONPathRecursionError("recursion limit exceeded", token=self.token)

    yield node

    if isinstance(node.value, dict):
        for name, val in node.value.items():
            if isinstance(val, (dict, list)):
                _node = JSONPathNode(
                    value=val,
                    location=node.location + (name,),
                )
                yield from _visit(_node, depth + 1)
    elif isinstance(node.value, list):
        for i, element in enumerate(node.value):
            if isinstance(element, (dict, list)):
                _node = JSONPathNode(
                    value=element,
                    location=node.location + (i,),
                )
                yield from _visit(_node, depth + 1)


def resolve_selector(
    *,
    env: JSONPathEnvironment,
    root: JSONValue,
    node: JSONPathNode,
    selector: Selector_,
) -> Iterable[JSONPathNode]:
    match selector:
        case Selector.Name(name=name):
            with suppress(KeyError, TypeError):
                yield JSONPathNode(
                    value=node.value[name],
                    location=node.location + (name,),
                )
        case Selector.Index(index=index):
            with suppress(IndexError, TypeError, KeyError):
                # TODO: normalize index
                yield JSONPathNode(
                    value=node.value[index],
                    location=node.location + (index,),
                )
        case Selector.Slice(start=start, stop=stop, step=step):
            if step != 0:
                i = start or 0
                _step = step or 1
                with suppress(ValueError, TypeError, KeyError):
                    for element in node.value[start:stop:_step]:
                        # TODO: normalize index
                        yield JSONPathNode(
                            value=element,
                            location=node.location + (i,),
                        )
                        i += _step
        case Selector.Wild():
            if isinstance(node.value, dict):
                for name, value in node.value.items():
                    yield JSONPathNode(
                        value=value,
                        location=node.location + (name,),
                    )
            elif isinstance(node.value, list):
                for i, element in enumerate(node.value):
                    yield JSONPathNode(
                        value=element,
                        location=node.location + (i,),
                    )
        case Selector.Filter(expression=expression):
            if isinstance(node.value, dict):
                for name, value in node.value.items():
                    if evaluate_filter(
                        env=env,
                        expression=expression,
                        root=root,
                        current=value,
                    ):
                        yield JSONPathNode(
                            value=value,
                            location=node.location + (name,),
                        )
            elif isinstance(node.value, list):
                for i, element in enumerate(node.value):
                    if evaluate_filter(
                        env=env,
                        expression=expression,
                        root=root,
                        current=element,
                    ):
                        yield JSONPathNode(
                            value=element,
                            location=node.location + (i,),
                        )
        case _:
            # TODO: suitable error
            raise ValueError(f":( {selector!r}")


def evaluate_filter(
    *,
    env: JSONPathEnvironment,
    expression: FilterExpression_,
    root: JSONValue,
    current: JSONValue,
) -> bool:
    return _is_truthy(
        evaluate_filter_expression(
            env=env,
            expression=expression,
            root=root,
            current=current,
        )
    )


def evaluate_filter_expression(
    *,
    env: JSONPathEnvironment,
    expression: FilterExpression_,
    root: JSONValue,
    current: JSONValue,
) -> object:
    match expression:
        case FilterExpression.True_():
            return True
        case FilterExpression.False_():
            return False
        case FilterExpression.Null():
            return None
        case FilterExpression.String(value=value):
            return value
        case FilterExpression.Int(value=value):
            return value
        case FilterExpression.Float(value=value):
            return value
        case FilterExpression.Not(expression=expression):
            return not _is_truthy(
                evaluate_filter_expression(
                    env=env,
                    expression=expression,
                    root=root,
                    current=current,
                )
            )
        case FilterExpression.Comparison(left=left, operator=op, right=right):
            _left = evaluate_filter_expression(
                env=env,
                expression=left,
                root=root,
                current=current,
            )
            if isinstance(_left, JSONPathNodeList) and len(_left) == 1:
                _left = _left[0].value

            _right = evaluate_filter_expression(
                env=env,
                expression=right,
                root=root,
                current=current,
            )
            if isinstance(_right, JSONPathNodeList) and len(_right) == 1:
                _right = _right[0].value

            return _compare(_left, op, _right)
        case FilterExpression.Logical(left=left, operator=op, right=right):
            return _logical(
                evaluate_filter_expression(
                    env=env,
                    expression=left,
                    root=root,
                    current=current,
                ),
                op,
                evaluate_filter_expression(
                    env=env,
                    expression=right,
                    root=root,
                    current=current,
                ),
            )
        case FilterExpression.RelativeQuery(query=query):
            return JSONPathNodeList(JSONPathQuery(query, env=env).finditer(current))
        case FilterExpression.RootQuery(query=query):
            return JSONPathNodeList(JSONPathQuery(query, env=env).finditer(root))
        case FilterExpression.Function(name=name, args=args):
            try:
                func = env.function_extensions[name]
            except KeyError:
                return NOTHING

            _args = [
                evaluate_filter_expression(
                    env=env, expression=arg, root=root, current=current
                )
                for arg in args
            ]

            return func(*_unpack_node_lists(func, _args))
        case _:
            raise ValueError(":(")  # TODO:


def _is_truthy(obj: object) -> bool:
    """Test for truthiness when evaluating filter expressions."""
    if isinstance(obj, JSONPathNodeList) and len(obj) == 0:
        return False
    if obj is NOTHING:
        return False
    if obj is None:
        return True
    return bool(obj)


def _logical(left: object, operator: LogicalOperator, right: object) -> bool:
    match operator:
        case LogicalOperator.And:
            return _is_truthy(left) and _is_truthy(right)
        case LogicalOperator.Or:
            return _is_truthy(left) or _is_truthy(right)
        case _:
            raise ValueError("unknown logical operator")  # TODO:


def _compare(  # noqa: PLR0911
    left: object, operator: ComparisonOperator, right: object
) -> bool:
    match operator:
        case ComparisonOperator.Eq:
            return _eq(left, right)
        case ComparisonOperator.Ne:
            return not _eq(left, right)
        case ComparisonOperator.Lt:
            return _lt(left, right)
        case ComparisonOperator.Gt:
            return _lt(right, left)
        case ComparisonOperator.Ge:
            return _lt(right, left) or _eq(left, right)
        case ComparisonOperator.Le:
            return _lt(left, right) or _eq(left, right)
        case _:
            raise ValueError("unknown comparison operator")  # TODO:


def _eq(left: object, right: object) -> bool:  # noqa: PLR0911
    if isinstance(right, JSONPathNodeList):
        left, right = right, left

    if isinstance(left, JSONPathNodeList):
        if isinstance(right, JSONPathNodeList):
            return left == right
        if left.empty():
            return right is NOTHING
        if len(left) == 1:
            return left[0] == right
        return False

    if left is NOTHING and right is NOTHING:
        return True

    # Remember 1 == True and 0 == False in Python
    if isinstance(right, bool):
        left, right = right, left

    if isinstance(left, bool):
        return isinstance(right, bool) and left == right

    return left == right


def _lt(left: object, right: object) -> bool:
    if isinstance(left, str) and isinstance(right, str):
        return left < right

    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return left < right

    return False


def _unpack_node_lists(func: FilterFunction, args: list[object]) -> list[object]:
    _args: list[object] = []
    for idx, arg in enumerate(args):
        if func.arg_types[idx] != ExpressionType.NODES and isinstance(
            arg, JSONPathNodeList
        ):
            if len(arg) == 0:
                # If the query results in an empty nodelist, the
                # argument is the special result Nothing.
                _args.append(NOTHING)
            elif len(arg) == 1:
                # If the query results in a nodelist consisting of a
                # single node, the argument is the value of the node
                _args.append(arg[0].value)
            else:
                # This should not be possible as a non-singular query
                # would have been rejected when checking function
                # well-typedness.
                _args.append(arg)
        else:
            _args.append(arg)

    return _args
