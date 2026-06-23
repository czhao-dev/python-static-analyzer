"""SA001: High cyclomatic complexity."""

from __future__ import annotations

from c_static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA001"

_BRANCH_TYPES = {"if_statement", "for_statement", "while_statement", "do_statement", "conditional_expression"}
_BOOL_OPERATORS = {"&&", "||"}


def _walk(node):
    yield node
    for child in node.children:
        yield from _walk(child)


def _function_name(func_node) -> str:
    declarator = func_node.child_by_field_name("declarator")
    while declarator is not None and declarator.type != "function_declarator":
        declarator = declarator.child_by_field_name("declarator")
    if declarator is None:
        return "<anonymous>"
    name_node = declarator.child_by_field_name("declarator")
    return name_node.text.decode("utf-8") if name_node is not None else "<anonymous>"


def _score(node) -> int:
    score = 0
    if node.type in _BRANCH_TYPES:
        score += 1
    elif node.type == "case_statement" and node.child_by_field_name("value") is not None:
        score += 1
    elif node.type == "binary_expression":
        operator = node.child_by_field_name("operator")
        if operator is not None and operator.text.decode("utf-8") in _BOOL_OPERATORS:
            score += 1
    for child in node.children:
        score += _score(child)
    return score


def _compute_complexity(func_node) -> int:
    complexity = 1
    for child in func_node.children:
        complexity += _score(child)
    return complexity


def check(tree, source: bytes, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    threshold = config.max_complexity
    for node in _walk(tree.root_node):
        if node.type != "function_definition":
            continue
        score = _compute_complexity(node)
        if score > threshold:
            line, col = node.start_point[0] + 1, node.start_point[1]
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=line,
                    col=col,
                    rule_id=RULE_ID,
                    message=(
                        f"Function `{_function_name(node)}` has cyclomatic complexity {score} "
                        f"(threshold {threshold})"
                    ),
                )
            )
    return diagnostics
