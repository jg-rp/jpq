"""Base class for all function extensions."""

from abc import ABC
from abc import abstractmethod
from typing import Any

from .jpq import ExpressionType


class FilterFunction(ABC):
    """Base class for function extensions."""

    @property
    @abstractmethod
    def arg_types(self) -> list[ExpressionType]:
        """Argument types expected by the filter function."""

    @property
    @abstractmethod
    def return_type(self) -> ExpressionType:
        """The type of the value returned by the filter function."""

    @abstractmethod
    def __call__(self, *args: Any, **kwds: Any) -> Any:
        """Call the filter function."""
