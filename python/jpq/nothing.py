class Nothing:  # noqa: D100
    """The special result "Nothing"."""

    __slots__ = ()

    def __str__(self) -> str:
        return "<NOTHING>"

    def __repr__(self) -> str:
        return "<NOTHING>"


NOTHING = Nothing()
