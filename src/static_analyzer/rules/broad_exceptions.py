"""SA003: Broad exception handler."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA003"

_BROAD_NAMES = {"Exception", "BaseException"}


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.ExceptHandler):
            continue
        if node.type is None:
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=node.lineno,
                    col=node.col_offset,
                    rule_id=RULE_ID,
                    message="Broad exception handler `except:`",
                )
            )
        elif isinstance(node.type, ast.Name) and node.type.id in _BROAD_NAMES:
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=node.lineno,
                    col=node.col_offset,
                    rule_id=RULE_ID,
                    message=f"Broad exception handler `except {node.type.id}`",
                )
            )
    return diagnostics
