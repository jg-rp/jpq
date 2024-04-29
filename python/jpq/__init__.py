from .env import JSONPathEnvironment
from .env import JSONPathNodeList
from .env import JSONPathQuery
from .env import JSONValue
from .jpq import PyJSONPathError
from .nothing import Nothing
from .nothing import NOTHING

# TODO: __version__


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
compile = DEFAULT_ENV.compile
