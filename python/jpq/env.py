from __future__ import annotations
from typing import TYPE_CHECKING
from typing import Iterable

from .function_extensions import FilterFunction
from . import function_extensions
from .query import JSONPathQuery
from .query import JSONPathNodeList
from jpq import parse

if TYPE_CHECKING:
    from .query import JSONPathNode
    from .query import JSONValue


class JSONPathEnvironment:
    max_int_index = (2**53) - 1
    min_int_index = -(2**53) + 1
    max_recursion_depth = 100

    def __init__(self) -> None:
        self.function_extensions: dict[str, FilterFunction] = {}
        """A list of function extensions available to filters."""

        self.setup_function_extensions()

    def setup_function_extensions(self) -> None:
        """Initialize function extensions."""
        self.function_extensions["length"] = function_extensions.Length()
        self.function_extensions["count"] = function_extensions.Count()
        self.function_extensions["match"] = function_extensions.Match()
        self.function_extensions["search"] = function_extensions.Search()
        self.function_extensions["value"] = function_extensions.Value()

    def compile(self, query: str) -> JSONPathQuery:
        return JSONPathQuery(parse(query), env=self)

    def finditer(self, query: str, value: JSONValue) -> Iterable[JSONPathNode]:
        return self.compile(query).finditer(value)

    def find(self, query: str, value: JSONValue) -> JSONPathNodeList:
        return JSONPathNodeList(self.finditer(query, value))
