"""SA007: Deeply nested control flow."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA007"

_NESTED_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    threshold = config.max_nesting

    def maybe_report(node: ast.AST, depth: int, already_reported: list) -> None:
        if depth > threshold and not already_reported:
            already_reported.append(True)
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=node.lineno,
                    col=node.col_offset,
                    rule_id=RULE_ID,
                    message=f"Control flow nested {depth} levels deep (threshold {threshold})",
                )
            )

    def walk_if(stmt: ast.If, depth: int, reported: list) -> None:
        new_depth = depth + 1
        maybe_report(stmt, new_depth, reported)
        walk_block(stmt.body, new_depth, reported)
        if not stmt.orelse:
            return
        is_elif = len(stmt.orelse) == 1 and isinstance(stmt.orelse[0], ast.If) and (
            stmt.orelse[0].col_offset == stmt.col_offset
        )
        if is_elif:
            walk_if(stmt.orelse[0], depth, reported)  # elif chains don't add a nesting level
        else:
            walk_block(stmt.orelse, new_depth, reported)

    def walk_block(stmts: list[ast.stmt], depth: int, reported: list) -> None:
        for stmt in stmts:
            if isinstance(stmt, _NESTED_SCOPES):
                continue  # nested functions are scored separately
            if isinstance(stmt, ast.If):
                walk_if(stmt, depth, reported)
            elif isinstance(stmt, (ast.For, ast.AsyncFor, ast.While)):
                new_depth = depth + 1
                maybe_report(stmt, new_depth, reported)
                walk_block(stmt.body, new_depth, reported)
                walk_block(stmt.orelse, new_depth, reported)
            elif isinstance(stmt, (ast.With, ast.AsyncWith)):
                new_depth = depth + 1
                maybe_report(stmt, new_depth, reported)
                walk_block(stmt.body, new_depth, reported)
            elif isinstance(stmt, ast.Try):
                new_depth = depth + 1
                maybe_report(stmt, new_depth, reported)
                walk_block(stmt.body, new_depth, reported)
                for handler in stmt.handlers:
                    walk_block(handler.body, new_depth, reported)
                walk_block(stmt.orelse, new_depth, reported)
                walk_block(stmt.finalbody, new_depth, reported)

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            walk_block(node.body, 0, [])

    return diagnostics
