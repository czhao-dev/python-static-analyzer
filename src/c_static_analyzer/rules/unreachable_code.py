"""SA005: Unreachable code after return, break, continue, or goto."""

from __future__ import annotations

from c_static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA005"

_TERMINATOR_KEYWORDS = {
    "return_statement": "return",
    "break_statement": "break",
    "continue_statement": "continue",
    "goto_statement": "goto",
}


def _walk(node):
    yield node
    for child in node.children:
        yield from _walk(child)


def _case_body_stmts(case_stmt):
    value = case_stmt.child_by_field_name("value")
    return [c for c in case_stmt.named_children if c is not value]


def _check_block(stmts: list, path: str) -> list[Diagnostic]:
    for i, stmt in enumerate(stmts[:-1]):
        keyword = _TERMINATOR_KEYWORDS.get(stmt.type)
        if keyword is not None:
            unreachable = stmts[i + 1]
            line, col = unreachable.start_point[0] + 1, unreachable.start_point[1]
            return [
                Diagnostic(
                    path=path,
                    line=line,
                    col=col,
                    rule_id=RULE_ID,
                    message=f"Unreachable code after `{keyword}`",
                )
            ]
    return []


def check(tree, source: bytes, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for node in _walk(tree.root_node):
        if node.type == "compound_statement":
            diagnostics.extend(_check_block(list(node.named_children), path))
        elif node.type == "case_statement":
            diagnostics.extend(_check_block(_case_body_stmts(node), path))
    return diagnostics
