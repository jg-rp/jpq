"""JSONPath node and node list definitions."""

from __future__ import annotations

from typing import Any

JSONValue = list[Any] | dict[str, Any] | str | int | float | None | bool


class JSONPathNode:
    """A JSON-like value and its location in a JSON document.

    Attributes:
        value: The JSON-like value at this node.
        location: The names indices that make up the normalized path to _value_.
    """

    __slots__ = (
        "value",
        "location",
    )

    def __init__(
        self,
        *,
        value: Any,
        location: tuple[int | str, ...],
    ) -> None:
        self.value: Any = value
        self.location: tuple[int | str, ...] = location

    def path(self) -> str:
        """Return the normalized path to this node."""
        return "$" + "".join(
            (f"['{p}']" if isinstance(p, str) else f"[{p}]" for p in self.location)
        )

    def __str__(self) -> str:
        return f"JSONPathNode({self.path()!r})"


class JSONPathNodeList(list[JSONPathNode]):
    """A list JSONPathNode instances.

    This is a `list` subclass with some helper methods.
    """

    def values(self) -> list[object]:
        """Return the values from this node list."""
        return [node.value for node in self]

    def items(self) -> list[tuple[str, object]]:
        """Return a list of (path, value) pairs, one for each node in the list."""
        return [(node.path(), node.value) for node in self]

    def empty(self) -> bool:
        """Return `True` if this node list is empty, or `False` otherwise."""
        return not self

    def __str__(self) -> str:
        return f"NodeList{super().__str__()}"
