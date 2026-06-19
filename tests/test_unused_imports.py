from helpers import parse

from static_analyzer.rules import unused_imports


def test_flags_unused_import(config):
    tree = parse(
        """
import json
import os

print(os.getcwd())
"""
    )
    diagnostics = unused_imports.check(tree, "example.py", config)
    assert len(diagnostics) == 1
    assert "json" in diagnostics[0].message
    assert diagnostics[0].rule_id == "SA002"


def test_ignores_used_import_from(config):
    tree = parse(
        """
from pathlib import Path

p = Path(".")
"""
    )
    diagnostics = unused_imports.check(tree, "example.py", config)
    assert diagnostics == []


def test_respects_aliases(config):
    tree = parse(
        """
import numpy as np

print(np.array([1]))
"""
    )
    diagnostics = unused_imports.check(tree, "example.py", config)
    assert diagnostics == []


def test_ignores_future_imports(config):
    tree = parse(
        """
from __future__ import annotations
"""
    )
    diagnostics = unused_imports.check(tree, "example.py", config)
    assert diagnostics == []


def test_respects_dunder_all(config):
    tree = parse(
        """
from mymodule import helper

__all__ = ["helper"]
"""
    )
    diagnostics = unused_imports.check(tree, "example.py", config)
    assert diagnostics == []
