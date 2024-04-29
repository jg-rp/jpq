import json
import jpq

d = [{"a": [1, 2, 3]}, {"a": [1], "d": "f"}, {"a": 1, "d": "f"}]
q = "$[?count(@..*)>2]"

query = jpq.compile(q)
nodes = query.find(d)
assert jpq.find(q, d) == nodes
print(json.dumps([n for n, _ in nodes], indent=2))
print([n for n, _ in nodes])
print(len(nodes))
