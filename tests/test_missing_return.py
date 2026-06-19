from helpers import parse

from static_analyzer.rules import missing_return


def test_flags_missing_else_branch(config):
    tree = parse(
        """
def classify(x):
    if x > 0:
        return "positive"
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA008"


def test_ignores_complete_if_else(config):
    tree = parse(
        """
def classify(x):
    if x > 0:
        return "positive"
    else:
        return "non-positive"
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_function_without_return_value(config):
    tree = parse(
        """
def log(message):
    print(message)
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_try_except_that_always_exits(config):
    tree = parse(
        """
def parse(value):
    try:
        return int(value)
    except ValueError:
        return None
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_generator_functions(config):
    tree = parse(
        """
def gen(items):
    for item in items:
        if item:
            yield item
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_infinite_loop_without_break(config):
    tree = parse(
        """
def serve():
    while True:
        if should_stop():
            return "done"
        handle()
"""
    )
    diagnostics = missing_return.check(tree, "example.py", config)
    assert diagnostics == []
