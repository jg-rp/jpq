from __future__ import annotations

from enum import Enum

from .filter_function import FilterFunction

__all__ = (
    "FilterExpression",
    "Selector",
    "Segment",
    "Query",
    "Env",
    "PyJSONPathError",
)

class ComparisonOp(Enum):
    Eq = ...
    Ne = ...
    Ge = ...
    Gt = ...
    Le = ...
    Lt = ...

class LogicalOp(Enum):
    And = ...
    Or = ...

class FilterExpression:
    class True_:  # noqa: N801
        @property
        def span(self) -> tuple[int, int]: ...

    class False_:  # noqa: N801
        @property
        def span(self) -> tuple[int, int]: ...

    class Null:
        @property
        def span(self) -> tuple[int, int]: ...

    class String:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def value(self) -> str: ...

    class Int:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def value(self) -> int: ...

    class Float:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def value(self) -> float: ...

    class Not:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def expression(self) -> FilterExpression_: ...

    class Logical:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def left(self) -> FilterExpression_: ...
        @property
        def operator(self) -> LogicalOp: ...
        @property
        def right(self) -> FilterExpression_: ...

    class Comparison:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def left(self) -> FilterExpression_: ...
        @property
        def operator(self) -> ComparisonOp: ...
        @property
        def right(self) -> FilterExpression_: ...

    class RelativeQuery:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def query(self) -> Query: ...

    class RootQuery:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def query(self) -> Query: ...

    class Function:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def name(self) -> str: ...
        @property
        def args(self) -> list[FilterExpression_]: ...

FilterExpression_ = (
    FilterExpression.True_
    | FilterExpression.False_
    | FilterExpression.Null
    | FilterExpression.String
    | FilterExpression.Int
    | FilterExpression.Float
    | FilterExpression.Not
    | FilterExpression.Logical
    | FilterExpression.Comparison
    | FilterExpression.RelativeQuery
    | FilterExpression.RootQuery
    | FilterExpression.Function
)

class Selector:
    class Name:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def name(self) -> str: ...

    class Index:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def index(self) -> int: ...

    class Slice:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def start(self) -> int | None: ...
        @property
        def stop(self) -> int | None: ...
        @property
        def step(self) -> int | None: ...

    class Wild:
        @property
        def span(self) -> tuple[int, int]: ...

    class Filter:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def expression(self) -> FilterExpression_: ...

Selector_ = (
    Selector.Name | Selector.Index | Selector.Slice | Selector.Wild | Selector.Filter
)

SelectorList = list[Selector_]

class Segment:
    class Child:
        __match_args__ = ("selectors", "span")
        @property
        def selectors(self) -> SelectorList: ...
        @property
        def span(self) -> tuple[int, int]: ...

    class Recursive:
        __match_args__ = ("selectors", "span")
        @property
        def selectors(self) -> SelectorList: ...
        @property
        def span(self) -> tuple[int, int]: ...

class Query:
    @property
    def segments(self) -> list[Segment.Child | Segment.Recursive]: ...

Node = tuple[object, str]
NodeList = list[Node]

class Env:
    def __init__(
        self,
        function_register: dict[str, FilterFunction],
        nothing: object,
        *,
        strict: bool = True,
    ) -> None: ...
    def find(self, query: str, value: object) -> NodeList: ...
    def compile(self, query: str) -> Query: ...
    def query(self, query: Query, value: object) -> NodeList: ...

class PyJSONPathError(Exception): ...
class JSONPathTypeError(Exception): ...
class JSONPathSyntaxError(Exception): ...

class ExpressionType(Enum):
    """Function extension argument and return types."""

    Logical = ...
    Nodes = ...
    Value = ...
