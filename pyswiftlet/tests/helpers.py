import unittest
from typing import Iterable

from swiftlet import Swiftlet


ALGORITHMS = ("clr", "earley")


def normalize_inputs(texts: str | Iterable[str]) -> tuple[str, ...]:
    if isinstance(texts, str):
        return (texts,)
    return tuple(texts)


def make_parse_test(
    grammar: str,
    texts: str | Iterable[str],
    start: str,
    algorithms: tuple[str, ...] = ALGORITHMS,
):
    inputs = normalize_inputs(texts)

    def test(self: unittest.TestCase) -> None:
        for algorithm in algorithms:
            with self.subTest(algorithm=algorithm):
                parser = Swiftlet(grammar, start=start, algorithm=algorithm)
                for text in inputs:
                    with self.subTest(text=text):
                        ast = parser.parse(text)
                        self.assertIsNotNone(ast)

    return test
