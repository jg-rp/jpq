"""JSONPath configuration.

A JSONPathEnvironment is where you'd register functions extensions and
control recursion limits, for example.
"""

from __future__ import annotations

from typing import TYPE_CHECKING
from typing import Any

from .jpq import Env as _Env
from .jpq import Query as _Query
from .nothing import NOTHING

if TYPE_CHECKING:
    from . import FilterFunction


JSONValue = list[Any] | dict[str, Any] | str | int | float | None | bool
"""JSON-like data, as you would get from `json.load()`."""


class JSONPathEnvironment:
    """JSONPath configuration."""

    def __init__(self, *, strict: bool = True) -> None:
        self._function_extensions: dict[str, FilterFunction] = {}
        """A map of function extensions available to the filter selector."""

        self._strict = strict
        self.setup_function_extensions()
        self._env = _Env(self._function_extensions, NOTHING, strict=self._strict)

    def add_function_extension(self, name: str, ext: FilterFunction) -> None:
        """Add a JSONPath function extension."""
        self._function_extensions[name] = ext
        self._env = _Env(self._function_extensions, NOTHING, strict=self._strict)

    def setup_function_extensions(self) -> None:
        """Initialize function extensions."""
        from . import function_extensions

        self._function_extensions["length"] = function_extensions.Length()
        self._function_extensions["count"] = function_extensions.Count()
        self._function_extensions["match"] = function_extensions.Match()
        self._function_extensions["search"] = function_extensions.Search()
        self._function_extensions["value"] = function_extensions.Value()

    def compile(self, query: str) -> JSONPathQuery:
        """Prepare a JSONPath query ready for repeated application to different data."""
        return JSONPathQuery(self._env.compile(query), env=self._env)

    def find(self, query: str, value: JSONValue) -> JSONPathNodeList:
        """Apply the JSONPath expression _query_ to JSON-like data _value_."""
        return JSONPathNodeList(self._env.find(query, value))


class JSONPathQuery:
    """A compiled JSONPath query ready to be applied to JSON-like data."""

    __slots__ = ("_query", "_env")

    def __init__(self, query: _Query, env: _Env) -> None:
        self._query = query
        self._env = env

    def find(self, value: JSONValue) -> JSONPathNodeList:
        """Apply this query to JSON-like data _value_."""
        return JSONPathNodeList(self._env.query(self._query, value))


class JSONPathNodeList(list[tuple[object, str, int | str | None]]):
    """A list of (value, location, key) tuples resulting from applying a JSONPath
    query to some data.
    """  # noqa: D205

    def values(self) -> list[object]:
        """Return the values from this node list."""
        return [node[0] for node in self]

    def paths(self) -> list[str]:
        """Return a normalized path for each node in this node list."""
        return [node[1] for node in self]

    def keys(self) -> list[int | str | None]:
        """Return a name or index for each node in this node list."""
        return [node[2] for node in self]

    def empty(self) -> bool:
        """Return `True` if this node list is empty."""
        return not self

    def __str__(self) -> str:
        return f"JSONPathNodeList{super().__str__()}"
