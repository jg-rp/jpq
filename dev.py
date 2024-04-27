import json
from jpq import JSONPathEnvironment
from jpq import Env
from jpq.nothing import NOTHING

_env = JSONPathEnvironment()
fr = _env.function_extensions

d = {"a": "A", "b": "B"}
q = "$.a"

env = Env(fr, NOTHING)
query = env.compile(q)
nodes = env.find(q, d)
assert env.query(query, d) == nodes
print(json.dumps([n for n, _ in nodes], indent=2))
print([n for n, _ in nodes])
print(len(nodes))
