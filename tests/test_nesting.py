from helpers import parse

from static_analyzer.config import Config
from static_analyzer.rules import nesting


def test_shallow_nesting_is_not_flagged(config):
    tree = parse(
        """
def f(x):
    if x:
        return 1
    return 0
"""
    )
    diagnostics = nesting.check(tree, "example.py", config)
    assert diagnostics == []


def test_flags_deep_nesting():
    cfg = Config(max_nesting=2)
    tree = parse(
        """
def f(x):
    if x:
        for i in range(x):
            while i > 0:
                i -= 1
"""
    )
    diagnostics = nesting.check(tree, "example.py", cfg)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA007"


def test_elif_chain_does_not_count_as_nesting():
    cfg = Config(max_nesting=1)
    tree = parse(
        """
def f(x):
    if x == 1:
        return "one"
    elif x == 2:
        return "two"
    elif x == 3:
        return "three"
    else:
        return "other"
"""
    )
    diagnostics = nesting.check(tree, "example.py", cfg)
    assert diagnostics == []


def test_else_with_nested_if_does_count():
    cfg = Config(max_nesting=1)
    tree = parse(
        """
def f(x, y):
    if x:
        return 1
    else:
        if y:
            return 2
"""
    )
    diagnostics = nesting.check(tree, "example.py", cfg)
    assert len(diagnostics) == 1


def test_reports_only_once_per_function():
    cfg = Config(max_nesting=1)
    tree = parse(
        """
def f(x):
    if x:
        if x:
            if x:
                return 1
"""
    )
    diagnostics = nesting.check(tree, "example.py", cfg)
    assert len(diagnostics) == 1
