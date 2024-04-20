from .jpq import parse
from .jpq import FilterExpression
from .jpq import Selector
from .jpq import Segment

# TODO: we don't actually want to expose the above imports
# TODO: __version__

from .query import JSONPathQuery
from .query import JSONValue
from .env import JSONPathEnvironment

__all__ = (
    "FilterExpression",
    "JSONPathEnvironment",
    "JSONPathQuery",
    "Selector",
    "Segment",
    "parse",
    "find",
    "finditer",
    "compile",
    "JSONValue",
)

# For convenience
DEFAULT_ENV = JSONPathEnvironment()
compile = DEFAULT_ENV.compile  # noqa: A001
finditer = DEFAULT_ENV.finditer
find = DEFAULT_ENV.find
