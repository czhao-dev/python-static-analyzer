"""SA008: Missing return path in a function that appears to return a value."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA008"

_NESTED_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)


def _contains_break(stmts: list[ast.stmt]) -> bool:
    """Whether a `break` targeting THIS loop appears in stmts (not crossing nested loops/scopes)."""
    for stmt in stmts:
        if isinstance(stmt, ast.Break):
            return True
        if isinstance(stmt, _NESTED_SCOPES + (ast.For, ast.AsyncFor, ast.While)):
            continue
        for field in ("body", "orelse", "finalbody"):
            block = getattr(stmt, field, None)
            if isinstance(block, list) and _contains_break(block):
                return True
        for handler in getattr(stmt, "handlers", []):
            if _contains_break(handler.body):
                return True
    return False


def _always_exits(stmts: list[ast.stmt]) -> bool:
    """Whether executing this statement list always returns or raises (never falls through)."""
    if not stmts:
        return False
    return _stmt_always_exits(stmts[-1])


def _stmt_always_exits(stmt: ast.stmt) -> bool:
    if isinstance(stmt, (ast.Return, ast.Raise)):
        return True
    if isinstance(stmt, ast.If):
        if not stmt.orelse:
            return False
        return _always_exits(stmt.body) and _always_exits(stmt.orelse)
    if isinstance(stmt, (ast.With, ast.AsyncWith)):
        return _always_exits(stmt.body)
    if isinstance(stmt, ast.Try):
        if stmt.finalbody and _always_exits(stmt.finalbody):
            return True
        try_exits = _always_exits(stmt.orelse) if stmt.orelse else _always_exits(stmt.body)
        if stmt.handlers:
            return try_exits and all(_always_exits(h.body) for h in stmt.handlers)
        return try_exits
    if isinstance(stmt, ast.While):
        is_infinite = isinstance(stmt.test, ast.Constant) and bool(stmt.test.value) is True
        return is_infinite and not _contains_break(stmt.body)
    return False


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for func in ast.walk(tree):
        if not isinstance(func, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        if _is_generator(func):
            continue

        returns_value = any(
            isinstance(node, ast.Return) and node.value is not None
            for node in _iter_own_scope(func)
        )
        if returns_value and not _always_exits(func.body):
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=func.lineno,
                    col=func.col_offset,
                    rule_id=RULE_ID,
                    message=f"Function `{func.name}` may not return a value on all code paths",
                )
            )
    return diagnostics


def _iter_own_scope(node: ast.AST):
    for child in ast.iter_child_nodes(node):
        if isinstance(child, _NESTED_SCOPES):
            continue
        yield child
        yield from _iter_own_scope(child)


def _is_generator(func: ast.FunctionDef | ast.AsyncFunctionDef) -> bool:
    return any(isinstance(node, (ast.Yield, ast.YieldFrom)) for node in _iter_own_scope(func))
