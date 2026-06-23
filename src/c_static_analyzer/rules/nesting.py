"""SA003: Deeply nested control flow."""

from __future__ import annotations

from c_static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA003"

_LOOP_TYPES = {"for_statement", "while_statement", "do_statement"}


def _walk(node):
    yield node
    for child in node.children:
        yield from _walk(child)


def _case_body_stmts(case_stmt):
    value = case_stmt.child_by_field_name("value")
    return [c for c in case_stmt.named_children if c is not value]


def check(tree, source: bytes, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    threshold = config.max_nesting

    def maybe_report(node, depth: int, reported: list) -> None:
        if depth > threshold and not reported:
            reported.append(True)
            line, col = node.start_point[0] + 1, node.start_point[1]
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=line,
                    col=col,
                    rule_id=RULE_ID,
                    message=f"Control flow nested {depth} levels deep (threshold {threshold})",
                )
            )

    def walk_stmt_or_block(stmt, depth: int, reported: list) -> None:
        if stmt is None:
            return
        if stmt.type == "compound_statement":
            for child in stmt.named_children:
                walk_stmt(child, depth, reported)
        else:
            walk_stmt(stmt, depth, reported)

    def walk_if(stmt, depth: int, reported: list) -> None:
        new_depth = depth + 1
        maybe_report(stmt, new_depth, reported)
        walk_stmt_or_block(stmt.child_by_field_name("consequence"), new_depth, reported)

        alternative = stmt.child_by_field_name("alternative")
        if alternative is None:
            return
        inner = alternative.named_children[0] if alternative.named_children else None
        if inner is None:
            return
        if inner.type == "if_statement":
            walk_if(inner, depth, reported)  # elif chains don't add a nesting level
        else:
            walk_stmt_or_block(inner, new_depth, reported)

    def walk_stmt(stmt, depth: int, reported: list) -> None:
        if stmt.type == "if_statement":
            walk_if(stmt, depth, reported)
        elif stmt.type in _LOOP_TYPES:
            new_depth = depth + 1
            maybe_report(stmt, new_depth, reported)
            walk_stmt_or_block(stmt.child_by_field_name("body"), new_depth, reported)
        elif stmt.type == "switch_statement":
            new_depth = depth + 1
            maybe_report(stmt, new_depth, reported)
            body = stmt.child_by_field_name("body")
            if body is None:
                return
            for case_stmt in body.named_children:
                if case_stmt.type != "case_statement":
                    continue
                for sub in _case_body_stmts(case_stmt):
                    walk_stmt(sub, new_depth, reported)

    for func in _walk(tree.root_node):
        if func.type != "function_definition":
            continue
        body = func.child_by_field_name("body")
        if body is None:
            continue
        reported: list = []
        for stmt in body.named_children:
            walk_stmt(stmt, 0, reported)

    return diagnostics
