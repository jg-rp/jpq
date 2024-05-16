"""The standard `match` function extension."""

import regex as re
from iregexp_check import check

from jpq import ExpressionType
from jpq import FilterFunction

from ._pattern import map_re


class Match(FilterFunction):
    """The standard `match` function."""

    arg_types = [ExpressionType.Value, ExpressionType.Value]
    return_type = ExpressionType.Logical

    def __call__(self, string: str, pattern: object) -> bool:
        """Return `True` if _string_ matches _pattern_, or `False` otherwise."""
        # TODO: our own cache that includes the i-regexp check and map
        if not isinstance(pattern, str) or not check(pattern):
            return False

        try:
            # re.fullmatch caches compiled patterns internally
            return bool(re.fullmatch(map_re(pattern), string))
        except (TypeError, re.error):
            return False
