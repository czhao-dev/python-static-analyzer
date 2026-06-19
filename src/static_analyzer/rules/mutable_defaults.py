"""SA001: Mutable default argument."""

from __future__ import annotations

import ast

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA001"

_MUTABLE_TYPES = (ast.List, ast.Dict, ast.Set, ast.ListComp, ast.DictComp, ast.SetComp)


def _is_mutable(node: ast.expr) -> bool:
    if isinstance(node, _MUTABLE_TYPES):
        return True
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
        return node.func.id in {"list", "dict", "set"}
    return False


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        args = node.args
        positional = args.posonlyargs + args.args
        for arg, default in zip(reversed(positional), reversed(args.defaults)):
            if _is_mutable(default):
                diagnostics.append(
                    Diagnostic(
                        path=path,
                        line=default.lineno,
                        col=default.col_offset,
                        rule_id=RULE_ID,
                        message=f"Mutable default argument `{arg.arg}={ast.unparse(default)}`",
                    )
                )
        for arg, default in zip(args.kwonlyargs, args.kw_defaults):
            if default is not None and _is_mutable(default):
                diagnostics.append(
                    Diagnostic(
                        path=path,
                        line=default.lineno,
                        col=default.col_offset,
                        rule_id=RULE_ID,
                        message=f"Mutable default argument `{arg.arg}={ast.unparse(default)}`",
                    )
                )
    return diagnostics
