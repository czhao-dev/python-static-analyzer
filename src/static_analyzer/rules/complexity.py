"""SA005: High cyclomatic complexity."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA005"

_BRANCH_NODES = (
    ast.If,
    ast.For,
    ast.AsyncFor,
    ast.While,
    ast.ExceptHandler,
    ast.Assert,
    ast.IfExp,
)

_NESTED_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)


def _compute_complexity(func: ast.FunctionDef | ast.AsyncFunctionDef) -> int:
    complexity = 1

    def walk(node: ast.AST) -> None:
        nonlocal complexity
        for child in ast.iter_child_nodes(node):
            if isinstance(child, _NESTED_SCOPES) and child is not func:
                continue  # nested scopes are scored separately
            if isinstance(child, _BRANCH_NODES):
                complexity += 1
            elif isinstance(child, ast.comprehension):
                complexity += 1 + len(child.ifs)
            elif isinstance(child, ast.BoolOp):
                complexity += len(child.values) - 1
            walk(child)

    walk(func)
    return complexity


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    threshold = config.max_complexity
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        score = _compute_complexity(node)
        if score > threshold:
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=node.lineno,
                    col=node.col_offset,
                    rule_id=RULE_ID,
                    message=f"Function `{node.name}` has cyclomatic complexity {score} (threshold {threshold})",
                )
            )
    return diagnostics
