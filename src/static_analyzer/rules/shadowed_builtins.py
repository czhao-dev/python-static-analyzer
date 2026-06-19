"""SA004: Built-in name shadowed."""

from __future__ import annotations

import ast
import builtins

from static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA004"

_BUILTIN_NAMES = frozenset(dir(builtins))


def check(tree: ast.Module, path: str, config) -> list[Diagnostic]:
    diagnostics = []

    def report(name: str, lineno: int, col: int, kind: str) -> None:
        if name in _BUILTIN_NAMES:
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=lineno,
                    col=col,
                    rule_id=RULE_ID,
                    message=f"{kind} `{name}` shadows a built-in name",
                )
            )

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            report(node.name, node.lineno, node.col_offset, "Function")
            for arg in (*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs):
                report(arg.arg, arg.lineno, arg.col_offset, "Parameter")
            if node.args.vararg:
                report(node.args.vararg.arg, node.args.vararg.lineno, node.args.vararg.col_offset, "Parameter")
            if node.args.kwarg:
                report(node.args.kwarg.arg, node.args.kwarg.lineno, node.args.kwarg.col_offset, "Parameter")
        elif isinstance(node, ast.ClassDef):
            report(node.name, node.lineno, node.col_offset, "Class")
        elif isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name):
                    report(target.id, target.lineno, target.col_offset, "Variable")

    return diagnostics
