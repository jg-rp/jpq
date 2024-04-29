"""The standard `value` function extension."""

from __future__ import annotations

from typing import TYPE_CHECKING

from jpq import NOTHING
from jpq.function_extensions import ExpressionType
from jpq.function_extensions import FilterFunction

if TYPE_CHECKING:
    from jpq import NodeList


class Value(FilterFunction):
    """The standard `value` function."""

    arg_types = [ExpressionType.NODES]
    return_type = ExpressionType.VALUE

    def __call__(self, nodes: NodeList) -> object:
        """Return the first node in a node list if it has only one item."""
        if len(nodes) == 1:
            return nodes[0][0]
        return NOTHING
