"""SA002: Unused import."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA002"


def _collect_used_names(tree: ast.Module) -> set[str]:
    used = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            used.add(node.id)
        elif isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name):
            used.add(node.value.id)
    return used


def _dunder_all_entries(node: ast.Assign) -> list[str]:
    if not any(isinstance(target, ast.Name) and target.id == "__all__" for target in node.targets):
        return []
    if not isinstance(node.value, (ast.List, ast.Tuple)):
        return []
    return [elt.value for elt in node.value.elts if isinstance(elt, ast.Constant) and isinstance(elt.value, str)]


def _collect_dunder_all(tree: ast.Module) -> set[str]:
    names = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            names.update(_dunder_all_entries(node))
    return names


def _report(path: str, node: ast.stmt, import_name: str) -> Diagnostic:
    return Diagnostic(
        path=path,
        line=node.lineno,
        col=node.col_offset,
        rule_id=RULE_ID,
        message=f"Unused import `{import_name}`",
    )


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    used = _collect_used_names(tree) | _collect_dunder_all(tree)

    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                bound = alias.asname or alias.name.split(".")[0]
                if bound not in used:
                    diagnostics.append(_report(path, node, alias.name))
        elif isinstance(node, ast.ImportFrom) and node.module != "__future__":
            for alias in node.names:
                if alias.name == "*":
                    continue
                bound = alias.asname or alias.name
                if bound not in used:
                    diagnostics.append(_report(path, node, alias.name))
    return diagnostics
