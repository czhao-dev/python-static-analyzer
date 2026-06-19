from helpers import parse

from static_analyzer.rules import broad_exceptions


def test_flags_bare_except(config):
    tree = parse(
        """
try:
    risky()
except:
    pass
"""
    )
    diagnostics = broad_exceptions.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA003"


def test_flags_except_exception(config):
    tree = parse(
        """
try:
    risky()
except Exception:
    pass
"""
    )
    diagnostics = broad_exceptions.check(tree, "example.py", config)
    assert len(diagnostics) == 1


def test_ignores_specific_exception(config):
    tree = parse(
        """
try:
    risky()
except ValueError:
    pass
"""
    )
    diagnostics = broad_exceptions.check(tree, "example.py", config)
    assert diagnostics == []
