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
    Case(
        description="filter keys from an object",
        query="$.some[~?match(@, '^b.*')]",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=["thing"],
    ),
    Case(
        description="singular key from an object",
        query="$.some[~'other']",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=["other"],
    ),
    Case(
        description="singular key from an object, does not exist",
        query="$.some[~'else']",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=[],
    ),
    Case(
        description="singular key from an array",
        query="$.some[~'1']",
        data={"some": ["foo", "bar"]},
        want=[],
    ),
    Case(
        description="singular key from an object, shorthand",
        query="$.some.~other",
        data={"some": {"other": "foo", "thing": "bar"}},
        want=["other"],
    ),
    Case(
        description="recursive key from an object",
        query="$.some..[~'other']",
        data={"some": {"other": "foo", "thing": "bar", "else": {"other": "baz"}}},
        want=["other", "other"],
    ),
    Case(
        description="recursive key from an object, shorthand",
        query="$.some..~other",
        data={"some": {"other": "foo", "thing": "bar", "else": {"other": "baz"}}},
        want=["other", "other"],
    ),
    Case(
        description="recursive key from an object, does not exist",
        query="$.some..[~'nosuchthing']",
        data={"some": {"other": "foo", "thing": "bar", "else": {"other": "baz"}}},
        want=[],
    ),
]


@pytest.mark.parametrize("case", TEST_CASES, ids=operator.attrgetter("description"))
def test_non_standard(case: Case) -> None:
    nodes = find(case.query, case.data)
    assert nodes.values() == case.want


def test_location_of_keys_from_array() -> None:
    """Test the normalized path generated from the keys selector is a valid query."""
    query = "$.some.~"
    data = {"some": {"a": 1, "b": 2, "c": 3}}
    nodes = find(query, data)
    assert nodes.values() == ["a", "b", "c"]
    assert find(nodes[0][1], data) == [("a", "$['some'][~'a']")]


# TODO: Port docs example test cases
