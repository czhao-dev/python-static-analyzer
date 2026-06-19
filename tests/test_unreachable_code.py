from helpers import parse

from static_analyzer.rules import unreachable_code


def test_flags_code_after_return(config):
    tree = parse(
        """
def f():
    return 1
    print("never runs")
"""
    )
    diagnostics = unreachable_code.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA009"
    assert diagnostics[0].line == 4


def test_flags_code_after_break_in_loop(config):
    tree = parse(
        """
def f():
    for i in range(10):
        break
        print(i)
"""
    )
    diagnostics = unreachable_code.check(tree, "example.py", config)
    assert len(diagnostics) == 1


def test_ignores_reachable_code(config):
    tree = parse(
        """
def f(x):
    if x:
        return 1
    return 2
"""
    )
    diagnostics = unreachable_code.check(tree, "example.py", config)
    assert diagnostics == []
