from helpers import parse

from static_analyzer.rules import unused_variables


def test_flags_unused_local_variable(config):
    tree = parse(
        """
def compute():
    total = 0
    unused = 42
    return total
"""
    )
    diagnostics = unused_variables.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert "unused" in diagnostics[0].message
    assert diagnostics[0].rule_id == "SA006"


def test_ignores_used_variable(config):
    tree = parse(
        """
def compute():
    total = 0
    for item in range(10):
        total += item
    return total
"""
    )
    diagnostics = unused_variables.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_underscore_prefixed(config):
    tree = parse(
        """
def compute():
    _ignored = expensive_call()
    return 1
"""
    )
    diagnostics = unused_variables.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_global_declarations(config):
    tree = parse(
        """
counter = 0

def increment():
    global counter
    counter = counter + 1
"""
    )
    diagnostics = unused_variables.check(tree, "example.py", config)
    assert diagnostics == []
