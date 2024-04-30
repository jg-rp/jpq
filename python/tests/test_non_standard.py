"""Test cases for non-standard JSONPath syntax."""

import dataclasses
import operator

import pytest
from jpq import JSONValue
from jpq import find


@dataclasses.dataclass
class Case:
    description: str
    query: str
    data: JSONValue
    want: list[JSONValue]


TEST_CASES = [
    Case(
        description="keys from an object",
        query="$.some[~]",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=["other", "thing"],
    ),
    Case(
        description="shorthand keys from an object",
        query="$.some.~",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=["other", "thing"],
    ),
    Case(
        description="keys from an array",
        query="$.some[~]",
        data={"some": ["other", "thing"]},
        want=[],
    ),
    Case(
        description="shorthand keys from an array",
        query="$.some.~",
        data={"some": ["other", "thing"]},
        want=[],
    ),
    Case(
        description="recursive object keys",
        query="$..[~]",
        data={"some": {"thing": "else", "foo": {"bar": "baz"}}},
        want=["some", "thing", "foo", "bar"],
    ),
    Case(
        description="shorthand recursive object keys",
        query="$..~",
        data={"some": {"thing": "else", "foo": {"bar": "baz"}}},
        want=["some", "thing", "foo", "bar"],
    ),
    Case(
        description="current key of an object",
        query="$.some[?match(#, '^b.*')]",
        data={"some": {"foo": "a", "bar": "b", "baz": "c", "qux": "d"}},
        want=["b", "c"],
    ),
    Case(
        description="current key of an array",
        query="$.some[?# > 1]",
        data={"some": ["other", "thing", "foo", "bar"]},
        want=["foo", "bar"],
    ),
]


@pytest.mark.parametrize("case", TEST_CASES, ids=operator.attrgetter("description"))
def test_non_standard(case: Case) -> None:
    nodes = find(case.query, case.data)
    assert nodes.values() == case.want
