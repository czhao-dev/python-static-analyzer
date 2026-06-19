from helpers import parse

from static_analyzer.config import Config
from static_analyzer.rules import complexity


def test_simple_function_is_not_flagged(config):
    tree = parse(
        """
def add(a, b):
    return a + b
"""
    )
    diagnostics = complexity.check(tree, "example.py", config)
    assert diagnostics == []


def test_flags_high_complexity_function():
    cfg = Config(max_complexity=3)
    tree = parse(
        """
def classify(x):
    if x > 0:
        if x > 10:
            return "big"
        return "small"
    elif x < 0:
        return "negative"
    return "zero"
"""
    )
    diagnostics = complexity.check(tree, "example.py", cfg)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA005"


def test_nested_function_scored_independently():
    cfg = Config(max_complexity=1)
    tree = parse(
        """
def outer():
    def inner():
        if True:
            return 1
        return 2
    return inner()
"""
    )
    diagnostics = complexity.check(tree, "example.py", cfg)
    assert {d.message.split("`")[1] for d in diagnostics} == {"inner"}
