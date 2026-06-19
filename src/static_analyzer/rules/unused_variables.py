"""SA006: Unused local variable."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA006"

_NESTED_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)


def _assigned_names(target: ast.expr):
    if isinstance(target, ast.Name):
        yield target
    elif isinstance(target, (ast.Tuple, ast.List)):
        for elt in target.elts:
            yield from _assigned_names(elt)
    elif isinstance(target, ast.Starred):
        yield from _assigned_names(target.value)


def _iter_own_scope(node: ast.AST):
    """Yield descendant nodes without crossing into nested function/class scopes."""
    for child in ast.iter_child_nodes(node):
        if isinstance(child, _NESTED_SCOPES):
            continue
        yield child
        yield from _iter_own_scope(child)


def _collect_usage(func: ast.AST) -> tuple[set[str], set[str]]:
    used: set[str] = set()
    declared_global: set[str] = set()
    for node in ast.walk(func):
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Load):
            used.add(node.id)
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            declared_global.update(node.names)
        elif isinstance(node, ast.AugAssign) and isinstance(node.target, ast.Name):
            used.add(node.target.id)
    return used, declared_global


def _collect_first_assignments(func: ast.AST) -> dict[str, ast.Name]:
    first_assignment: dict[str, ast.Name] = {}
    for node in _iter_own_scope(func):
        if not isinstance(node, ast.Assign):
            continue
        candidates = (n for target in node.targets for n in _assigned_names(target) if not n.id.startswith("_"))
        for name_node in candidates:
            first_assignment.setdefault(name_node.id, name_node)
    return first_assignment


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for func in ast.walk(tree):
        if not isinstance(func, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue

        used, declared_global = _collect_usage(func)
        first_assignment = _collect_first_assignments(func)

        for name, name_node in first_assignment.items():
            if name in used or name in declared_global:
                continue
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=name_node.lineno,
                    col=name_node.col_offset,
                    rule_id=RULE_ID,
                    message=f"Local variable `{name}` is assigned but never used",
                )
            )

    return diagnostics
