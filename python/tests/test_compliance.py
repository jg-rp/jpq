"""Test Python JSONPath against the JSONPath Compliance Test Suite.

The CTS is a submodule located in /tests/cts. After a git clone, run
`git submodule update --init` from the root of the repository.
"""

import json
import operator
from dataclasses import dataclass
from typing import Any

import jpq
import pytest
from jpq import JSONValue


@dataclass
class Case:
    name: str
    selector: str
    document: JSONValue = None
    result: Any = None
    results: list[Any] | None = None
    invalid_selector: bool | None = None


SKIP: dict[str, str] = {}


def cases() -> list[Case]:
    with open("python/tests/cts/cts.json", encoding="utf8") as fd:
        data = json.load(fd)
    return [Case(**case) for case in data["tests"]]


def valid_cases() -> list[Case]:
    return [case for case in cases() if not case.invalid_selector]


def invalid_cases() -> list[Case]:
    return [case for case in cases() if case.invalid_selector]


@pytest.mark.parametrize("case", valid_cases(), ids=operator.attrgetter("name"))
def test_compliance(case: Case) -> None:
    if case.name in SKIP:
        pytest.skip(reason=SKIP[case.name])  # no cov

    assert case.document is not None
    rv = [
        n
        for n, _ in jpq.find(
            case.selector,
            case.document,
        )
    ]

    if case.results is not None:
        assert rv in case.results
    else:
        assert rv == case.result


@pytest.mark.parametrize("case", invalid_cases(), ids=operator.attrgetter("name"))
def test_invalid_selectors(case: Case) -> None:
    if case.name in SKIP:
        pytest.skip(reason=SKIP[case.name])  # no cov

    with pytest.raises(jpq.PyJSONPathError):
        jpq.compile(case.selector)
