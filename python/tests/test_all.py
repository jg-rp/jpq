import pytest
import jpq


def test_sum_as_string():
    assert jpq.sum_as_string(1, 1) == "2"
