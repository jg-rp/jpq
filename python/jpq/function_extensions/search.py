"""The standard `search` function extension."""

import regex as re

from jpq import ExpressionType
from jpq import FilterFunction


class Search(FilterFunction):
    """The standard `search` function."""

    arg_types = [ExpressionType.Value, ExpressionType.Value]
    return_type = ExpressionType.Logical

    def __call__(self, string: str, pattern: str) -> bool:
        """Return `True` if _string_ contains _pattern_, or `False` otherwise."""
        try:
            # re.search caches compiled patterns internally
            return bool(re.search(pattern, string))
        except (TypeError, re.error):
            return False
