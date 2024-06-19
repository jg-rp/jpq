from .jpq import ExpressionType
from .jpq import JSONPathSyntaxError
from .jpq import JSONPathTypeError
from .jpq import Node
from .jpq import PyJSONPathError

NodeList = list[Node]

from .env import JSONPathEnvironment
from .env import JSONPathNodeList
from .env import JSONPathQuery
from .env import JSONValue
from .filter_function import FilterFunction
from .nothing import NOTHING
from .nothing import Nothing

__all__ = (
    "Env",
    "ExpressionType",
    "FilterFunction",
    "JSONPathEnvironment",
    "JSONPathNodeList",
    "JSONPathQuery",
    "JSONValue",
    "Nothing",
    "NOTHING",
    "PyJSONPathError",
    "JSONPathSyntaxError",
    "JSONPathTypeError",
)


DEFAULT_ENV = JSONPathEnvironment()
find = DEFAULT_ENV.find
compile = DEFAULT_ENV.compile  # noqa: A001
