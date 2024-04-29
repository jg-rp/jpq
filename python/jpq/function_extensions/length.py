"""The standard `length` function extension."""

from collections.abc import Sized

from jpq import NOTHING
from jpq import ExpressionType
from jpq import FilterFunction
from jpq import Nothing


class Length(FilterFunction):
    """The standard `length` function."""

    arg_types = [ExpressionType.Value]
    return_type = ExpressionType.Value

    def __call__(self, obj: Sized) -> int | Nothing:
        """Return an object's length.

        If the object does not have a length, the special _Nothing_ value is
        returned.
        """
        try:
            return len(obj)
        except TypeError:
            return NOTHING
