from .node import JSONPathNodeList


class Nothing:
    """The special result "Nothing"."""

    __slots__ = ()

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Nothing) or (
            isinstance(other, JSONPathNodeList) and other.empty()
        )

    def __str__(self) -> str:
        return "<NOTHING>"

    def __repr__(self) -> str:
        return "<NOTHING>"


NOTHING = Nothing()
