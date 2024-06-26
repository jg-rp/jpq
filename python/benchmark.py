import json
import timeit
from typing import Any
from typing import Mapping
from typing import NamedTuple
from typing import Sequence
from typing import Union

# ruff: noqa: D100 D101 D103 T201


class CTSCase(NamedTuple):
    query: str
    data: Union[Sequence[Any], Mapping[str, Any]]


def valid_queries() -> Sequence[CTSCase]:
    with open("python/tests/cts/cts.json") as fd:
        data = json.load(fd)

    return [
        (CTSCase(t["selector"], t["document"]))
        for t in data["tests"]
        if not t.get("invalid_selector", False)
    ]


QUERIES = valid_queries()

COMPILE_AND_FIND_SETUP = "from jpq import find"

COMPILE_AND_FIND_STMT = """\
for path, data in QUERIES:
    find(path, data)"""

COMPILE_AND_FIND_VALUES_STMT = """\
for path, data in QUERIES:
    find(path, data).values()"""

JUST_COMPILE_SETUP = "from jpq import compile"

JUST_COMPILE_STMT = """\
for path, _ in QUERIES:
    compile(path)"""

JUST_FIND_SETUP = """\
from jpq import compile
compiled_queries = [(compile(q), d) for q, d in QUERIES]
"""

JUST_FIND_STMT = """\
for query, data in compiled_queries:
    query.find(data)"""

JUST_FIND_VALUES_STMT = """\
for query, data in compiled_queries:
    [node.value for node in query.find(data)]"""


def benchmark(number: int = 100, best_of: int = 3) -> None:
    print(f"repeating {len(QUERIES)} queries {number} times, best of {best_of} rounds")

    results = timeit.repeat(
        COMPILE_AND_FIND_STMT,
        setup=COMPILE_AND_FIND_SETUP,
        globals={"QUERIES": QUERIES},
        number=number,
        repeat=best_of,
    )

    print("compile and find".ljust(30), f"\033[92m{min(results):.3f}\033[0m")

    results = timeit.repeat(
        COMPILE_AND_FIND_VALUES_STMT,
        setup=COMPILE_AND_FIND_SETUP,
        globals={"QUERIES": QUERIES},
        number=number,
        repeat=best_of,
    )

    print("compile and find (values)".ljust(30), f"{min(results):.3f}")

    results = timeit.repeat(
        JUST_COMPILE_STMT,
        setup=JUST_COMPILE_SETUP,
        globals={"QUERIES": QUERIES},
        number=number,
        repeat=best_of,
    )

    print("just compile".ljust(30), f"{min(results):.3f}")

    results = timeit.repeat(
        JUST_FIND_STMT,
        setup=JUST_FIND_SETUP,
        globals={"QUERIES": QUERIES},
        number=number,
        repeat=best_of,
    )

    print("just find".ljust(30), f"\033[92m{min(results):.3f}\033[0m")

    results = timeit.repeat(
        JUST_FIND_VALUES_STMT,
        setup=JUST_FIND_SETUP,
        globals={"QUERIES": QUERIES},
        number=number,
        repeat=best_of,
    )

    print("just find (values)".ljust(30), f"{min(results):.3f}")


if __name__ == "__main__":
    benchmark()
