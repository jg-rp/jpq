"""The standard `value` function extension."""

from __future__ import annotations

from typing import TYPE_CHECKING

from jpq import NOTHING
from jpq import ExpressionType
from jpq import FilterFunction

if TYPE_CHECKING:
    from jpq import NodeList


class Value(FilterFunction):
    """The standard `value` function."""

    arg_types = [ExpressionType.Nodes]
    return_type = ExpressionType.Value

    def __call__(self, nodes: NodeList) -> object:
        """Return the first node in a node list if it has only one item."""
        if len(nodes) == 1:
            return nodes[0].value
        return NOTHING
