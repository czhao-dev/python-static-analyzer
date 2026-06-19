from helpers import parse

from static_analyzer.rules import mutable_defaults


def test_flags_list_default(config):
    tree = parse(
        """
def add_item(item, items=[]):
    items.append(item)
    return items
"""
    )
    diagnostics = mutable_defaults.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA001"
    assert "items=[]" in diagnostics[0].message


def test_flags_dict_and_set_defaults(config):
    tree = parse(
        """
def merge(data={}, tags=set()):
    return data, tags
"""
    )
    diagnostics = mutable_defaults.check(tree, "example.py", config)
    assert len(diagnostics) == 2


def test_ignores_immutable_defaults(config):
    tree = parse(
        """
def add_item(item, items=None, count=0, name=""):
    return item
"""
    )
    diagnostics = mutable_defaults.check(tree, "example.py", config)
    assert diagnostics == []


def test_flags_kwonly_mutable_default(config):
    tree = parse(
        """
def configure(*, options=[]):
    return options
"""
    )
    diagnostics = mutable_defaults.check(tree, "example.py", config)
    assert len(diagnostics) == 1
