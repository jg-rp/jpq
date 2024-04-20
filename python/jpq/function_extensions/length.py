"""The standard `length` function extension."""

from collections.abc import Sized

from jpq.nothing import NOTHING
from jpq.nothing import Nothing
from jpq.function_extensions import ExpressionType
from jpq.function_extensions import FilterFunction


class Length(FilterFunction):
    """The standard `length` function."""

    arg_types = [ExpressionType.VALUE]
    return_type = ExpressionType.VALUE

    def __call__(self, obj: Sized) -> int | Nothing:
        """Return an object's length.

        If the object does not have a length, the special _Nothing_ value is
        returned.
        """
        try:
            return len(obj)
        except TypeError:
            return NOTHING
