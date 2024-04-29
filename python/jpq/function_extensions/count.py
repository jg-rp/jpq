"""The standard `count` function extension."""

from __future__ import annotations

from typing import TYPE_CHECKING

from jpq import ExpressionType
from jpq import FilterFunction

if TYPE_CHECKING:
    from jpq import NodeList


class Count(FilterFunction):
    """The built-in `count` function."""

    arg_types = [ExpressionType.Nodes]
    return_type = ExpressionType.Value

    def __call__(self, node_list: NodeList) -> int:
        """Return the number of nodes in the node list."""
        return len(node_list)
