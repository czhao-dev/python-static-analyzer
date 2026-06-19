"""SA009: Unreachable code after return, raise, break, or continue."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA009"

_TERMINATORS = (ast.Return, ast.Raise, ast.Break, ast.Continue)
_KEYWORD = {
    ast.Return: "return",
    ast.Raise: "raise",
    ast.Break: "break",
    ast.Continue: "continue",
}


def _check_block(stmts: list[ast.stmt], path: str) -> list[Diagnostic]:
    for i, stmt in enumerate(stmts[:-1]):
        if isinstance(stmt, _TERMINATORS):
            unreachable = stmts[i + 1]
            keyword = _KEYWORD[type(stmt)]
            return [
                Diagnostic(
                    path=path,
                    line=unreachable.lineno,
                    col=unreachable.col_offset,
                    rule_id=RULE_ID,
                    message=f"Unreachable code after `{keyword}`",
                )
            ]
    return []


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for node in ast.walk(tree):
        for attr in ("body", "orelse", "finalbody"):
            block = getattr(node, attr, None)
            if isinstance(block, list) and block and isinstance(block[0], ast.stmt):
                diagnostics.extend(_check_block(block, path))
    return diagnostics
