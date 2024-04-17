__all__ = (
    "FilterExpressionType",
    "FilterExpression",
    "Selector",
    "Segment",
    "Query",
    "parse",
)

class FilterExpressionType:
    class True_:
        pass

    class False_:
        pass

    class Null:
        pass

    # TODO: finish me

class FilterExpression:
    @property
    def span(self) -> tuple[int, int]: ...
    @property
    def kind(self) -> FilterExpressionType: ...

class Selector:
    class Name:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def name(self) -> str: ...

    class Index:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def index(self) -> int: ...

    class Slice:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def start(self) -> int | None: ...
        @property
        def stop(self) -> int | None: ...
        @property
        def step(self) -> int | None: ...

    class Wild:
        @property
        def span(self) -> tuple[int, int]: ...

    class Filter:
        @property
        def span(self) -> tuple[int, int]: ...
        @property
        def expression(self) -> FilterExpression: ...

class Segment:
    @property
    def selectors(self) -> list[Selector]: ...
    @property
    def span(self) -> tuple[int, int]: ...

class Query:
    @property
    def segments(self) -> list[Segment]: ...

def parse(query: str) -> Query: ...
