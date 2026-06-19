from helpers import parse

from static_analyzer.rules import shadowed_builtins


def test_flags_shadowed_function_name(config):
    tree = parse(
        """
def list():
    return []
"""
    )
    diagnostics = shadowed_builtins.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA004"


def test_flags_shadowed_parameter(config):
    tree = parse(
        """
def process(id, dict):
    return id, dict
"""
    )
    diagnostics = shadowed_builtins.check(tree, "example.py", config)
    assert len(diagnostics) == 2


def test_flags_shadowed_variable_assignment(config):
    tree = parse(
        """
list = [1, 2, 3]
"""
    )
    diagnostics = shadowed_builtins.check(tree, "example.py", config)
    assert len(diagnostics) == 1


def test_ignores_non_builtin_names(config):
    tree = parse(
        """
def process(item, value):
    return item, value
"""
    )
    diagnostics = shadowed_builtins.check(tree, "example.py", config)
    assert diagnostics == []
