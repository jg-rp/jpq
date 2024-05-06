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


@dataclasses.dataclass
class DocsTestCase:
    description: str
    query: str
    data: JSONValue
    want: list[JSONValue]
    want_paths: list[str]


DOCS_TEST_CASES: list[DocsTestCase] = [
    DocsTestCase(
        description="key selector, key of nested object",
        query="$.a[0].~c",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=["c"],
        want_paths=["$['a'][0][~'c']"],
    ),
    DocsTestCase(
        description="key selector, key does not exist",
        query="$.a[1].~c",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=[],
        want_paths=[],
    ),
    DocsTestCase(
        description="key selector, descendant, single quoted key",
        query="$..[~'b']",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=["b", "b"],
        want_paths=["$['a'][0][~'b']", "$['a'][1][~'b']"],
    ),
    DocsTestCase(
        description="key selector, , descendant, double quoted key",
        query='$..[~"b"]',
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=["b", "b"],
        want_paths=["$['a'][0][~'b']", "$['a'][1][~'b']"],
    ),
    DocsTestCase(
        description="keys selector, object key",
        query="$.a[0].~",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=["b", "c"],
        want_paths=["$['a'][0][~'b']", "$['a'][0][~'c']"],
    ),
    DocsTestCase(
        description="keys selector, array key",
        query="$.a.~",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=[],
        want_paths=[],
    ),
    DocsTestCase(
        description="keys selector, descendant keys",
        query="$..[~]",
        data={
            "a": [{"b": "x", "c": "z"}, {"b": "y"}],
        },
        want=["a", "b", "c", "b"],
        want_paths=[
            "$[~'a']",
            "$['a'][0][~'b']",
            "$['a'][0][~'c']",
            "$['a'][1][~'b']",
        ],
    ),
    DocsTestCase(
        description="keys filter selector, conditionally select object keys",
        query="$.*[~?length(@) > 2]",
        data=[{"a": [1, 2, 3], "b": [4, 5]}, {"c": {"x": [1, 2]}}, {"d": [1, 2, 3]}],
        want=["a", "d"],
        want_paths=["$[0][~'a']", "$[2][~'d']"],
    ),
    DocsTestCase(
        description="keys filter selector, existence test",
        query="$.*[~?@.x]",
        data=[{"a": [1, 2, 3], "b": [4, 5]}, {"c": {"x": [1, 2]}}, {"d": [1, 2, 3]}],
        want=["c"],
        want_paths=["$[1][~'c']"],
    ),
    DocsTestCase(
        description="keys filter selector, keys from an array",
        query="$[~?(true == true)]",
        data=[{"a": [1, 2, 3], "b": [4, 5]}, {"c": {"x": [1, 2]}}, {"d": [1, 2, 3]}],
        want=[],
        want_paths=[],
    ),
    DocsTestCase(
        description="current key identifier, match on object names",
        query="$[?match(#, '^ab.*') && length(@) > 0 ]",
        data={"abc": [1, 2, 3], "def": [4, 5], "abx": [6], "aby": []},
        want=[[1, 2, 3], [6]],
        want_paths=["$['abc']", "$['abx']"],
    ),
    DocsTestCase(
        description="current key identifier, compare current array index",
        query="$.abc[?(# >= 1)]",
        data={"abc": [1, 2, 3], "def": [4, 5], "abx": [6], "aby": []},
        want=[2, 3],
        want_paths=["$['abc'][1]", "$['abc'][2]"],
    ),
]


@pytest.mark.parametrize(
    "case", DOCS_TEST_CASES, ids=operator.attrgetter("description")
)
def test_docs_examples(case: DocsTestCase) -> None:
    nodes = find(case.query, case.data)
    assert nodes.values() == case.want
    assert nodes.paths() == case.want_paths
