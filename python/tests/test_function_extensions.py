"""Function well-typedness test derived from IETF spec examples.

The test cases defined here are taken from version 11 of the JSONPath
internet draft, draft-ietf-jsonpath-base-11. In accordance with
https://trustee.ietf.org/license-info, Revised BSD License text
is included bellow.

See https://datatracker.ietf.org/doc/html/draft-ietf-jsonpath-base-20

Copyright (c) 2023 IETF Trust and the persons identified as authors
of the code. All rights reserved.Redistribution and use in source and
binary forms, with or without modification, are permitted provided
that the following conditions are met:

- Redistributions of source code must retain the above copyright
  notice, this list of conditions and the following disclaimer.
- Redistributions in binary form must reproduce the above copyright
  notice, this list of conditions and the following disclaimer in the
  documentation and/or other materials provided with the
  distribution.
- Neither the name of Internet Society, IETF or IETF Trust, nor the
  names of specific contributors, may be used to endorse or promote
  products derived from this software without specific prior written
  permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
“AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""  # noqa: D205

import dataclasses
import operator

import pytest
from jpq import ExpressionType
from jpq import FilterFunction
from jpq import JSONPathEnvironment
from jpq import JSONPathNodeList
from jpq import JSONPathTypeError


@dataclasses.dataclass
class Case:
    description: str
    query: str
    valid: bool


TEST_CASES = [
    Case(
        description="length, singular query, compared",
        query="$[?length(@) < 3]",
        valid=True,
    ),
    Case(
        description="length, non-singular query, compared",
        query="$[?length(@.*) < 3]",
        valid=False,
    ),
    Case(
        description="count, non-singular query, compared",
        query="$[?count(@.*) == 1]",
        valid=True,
    ),
    Case(
        description="count, int literal, compared",
        query="$[?count(1) == 1]",
        valid=False,
    ),
    Case(
        description="nested function, LogicalType -> NodesType",
        query="$[?count(foo(@.*)) == 1]",
        valid=True,
    ),
    Case(
        description="match, singular query, string literal",
        query="$[?match(@.timezone, 'Europe/.*')]",
        valid=True,
    ),
    Case(
        description="match, singular query, string literal, compared",
        query="$[?match(@.timezone, 'Europe/.*') == true]",
        valid=False,
    ),
    Case(
        description="value, non-singular query param, comparison",
        query="$[?value(@..color) == 'red']",
        valid=True,
    ),
    Case(
        description="value, non-singular query param",
        query="$[?value(@..color)]",
        valid=False,
    ),
    Case(
        description="function, singular query, value type param, logical return type",
        query="$[?bar(@.a)]",
        valid=True,
    ),
    Case(
        description=(
            "function, non-singular query, value type param, logical return type"
        ),
        query="$[?bar(@.*)]",
        valid=False,
    ),
    Case(
        description=(
            "function, non-singular query, nodes type param, logical return type"
        ),
        query="$[?bn(@.*)]",
        valid=True,
    ),
    Case(
        description=(
            "function, non-singular query, logical type param, logical return type"
        ),
        query="$[?bl(@.*)]",
        valid=True,
    ),
    Case(
        description="function, logical type param, comparison, logical return type",
        query="$[?bl(1==1)]",
        valid=True,
    ),
    Case(
        description="function, logical type param, literal, logical return type",
        query="$[?bl(1)]",
        valid=False,
    ),
    Case(
        description="function, value type param, literal, logical return type",
        query="$[?bar(1)]",
        valid=True,
    ),
]


class MockFoo(FilterFunction):
    arg_types = [ExpressionType.Nodes]
    return_type = ExpressionType.Nodes

    def __call__(self, nodes: JSONPathNodeList) -> JSONPathNodeList:  # noqa: D102
        return nodes  # no cov


class MockBar(FilterFunction):
    arg_types = [ExpressionType.Value]
    return_type = ExpressionType.Logical

    def __call__(self) -> bool:  # noqa: D102
        return False  # no cov


class MockBn(FilterFunction):
    arg_types = [ExpressionType.Nodes]
    return_type = ExpressionType.Logical

    def __call__(self, _: object) -> bool:  # noqa: D102
        return False  # no cov


class MockBl(FilterFunction):
    arg_types = [ExpressionType.Logical]
    return_type = ExpressionType.Logical

    def __call__(self, _: object) -> bool:  # noqa: D102
        return False  # no cov


@pytest.fixture()
def env() -> JSONPathEnvironment:
    environment = JSONPathEnvironment()
    environment.add_function_extension("foo", MockFoo())
    environment.add_function_extension("bar", MockBar())
    environment.add_function_extension("bn", MockBn())
    environment.add_function_extension("bl", MockBl())
    return environment


@pytest.mark.parametrize("case", TEST_CASES, ids=operator.attrgetter("description"))
def test_ietf_well_typedness(env: JSONPathEnvironment, case: Case) -> None:
    if case.valid:
        env.compile(case.query)
    else:
        with pytest.raises(JSONPathTypeError):
            env.compile(case.query)
