import pytest
from jpq import JSONPathEnvironment
from jpq import JSONPathSyntaxError


def test_strict_default() -> None:
    """Test that JSONPath syntax is strict by default."""
    env = JSONPathEnvironment()
    with pytest.raises(JSONPathSyntaxError, match=r"keys syntax \(`~`\) is disabled"):
        env.compile("$.~")


def test_not_strict() -> None:
    """Test that we can enable non-standard syntax."""
    env = JSONPathEnvironment(strict=False)
    query = env.compile("$.~")
    assert query.find({"a": [1, 2, 3]}).values() == ["a"]


# TODO: test bad function register
# TODO: test bad function (missing or incorrect signature)
