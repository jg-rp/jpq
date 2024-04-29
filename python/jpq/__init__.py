from .env import JSONPathEnvironment
from .env import JSONPathNodeList
from .env import JSONPathQuery
from .env import JSONValue
from .jpq import PyJSONPathError
from .nothing import NOTHING
from .nothing import Nothing

Node = tuple[object, str]
NodeList = list[Node]

__all__ = (
    "Env",
    "JSONPathEnvironment",
    "JSONPathNodeList",
    "JSONPathQuery",
    "JSONValue",
    "Nothing",
    "NOTHING",
    "PyJSONPathError",
)

DEFAULT_ENV = JSONPathEnvironment()
find = DEFAULT_ENV.find
compile = DEFAULT_ENV.compile  # noqa: A001
