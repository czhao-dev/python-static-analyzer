"""SA004: Missing return path in a non-void function."""

from __future__ import annotations

from c_static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA004"

_INFINITE_LOOP_TYPES = {"while_statement", "do_statement"}
_LOOP_OR_SWITCH_TYPES = {"for_statement", "while_statement", "do_statement", "switch_statement"}


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


def _is_void_function(func_node) -> bool:
    type_node = func_node.child_by_field_name("type")
    if type_node is None or type_node.text.decode("utf-8") != "void":
        return False
    declarator = func_node.child_by_field_name("declarator")
    while declarator is not None and declarator.type != "function_declarator":
        if declarator.type == "pointer_declarator":
            return False  # returns void*, which is a value
        declarator = declarator.child_by_field_name("declarator")
    return True


def _block_stmts(node) -> list:
    if node is None:
        return []
    if node.type == "compound_statement":
        return list(node.named_children)
    return [node]


def _contains_break(stmts: list) -> bool:
    """Whether a `break` targeting THIS loop/switch appears in stmts (not crossing a nested one)."""
    for stmt in stmts:
        if stmt.type == "break_statement":
            return True
        if stmt.type in _LOOP_OR_SWITCH_TYPES:
            continue  # break here targets the nested construct, not us
        if stmt.type == "compound_statement":
            if _contains_break(list(stmt.named_children)):
                return True
            continue
        for field in ("consequence", "body"):
            child = stmt.child_by_field_name(field)
            if child is not None and _contains_break(_block_stmts(child)):
                return True
        alternative = stmt.child_by_field_name("alternative")
        if alternative is not None:
            inner = alternative.named_children[0] if alternative.named_children else None
            if inner is not None and _contains_break(_block_stmts(inner)):
                return True
    return False


def _always_exits(node) -> bool:
    """Whether executing this statement (or block) always returns (never falls through)."""
    stmts = _block_stmts(node)
    if not stmts:
        return False
    return _stmt_always_exits(stmts[-1])


def _stmt_always_exits(stmt) -> bool:
    if stmt.type == "return_statement":
        return True
    if stmt.type == "if_statement":
        alternative = stmt.child_by_field_name("alternative")
        if alternative is None:
            return False
        inner = alternative.named_children[0] if alternative.named_children else None
        if inner is None:
            return False
        return _always_exits(stmt.child_by_field_name("consequence")) and _always_exits(inner)
    if stmt.type in _INFINITE_LOOP_TYPES:
        condition = stmt.child_by_field_name("condition")
        is_infinite = condition is not None and condition.text.decode("utf-8").strip("() ") in {"1", "true"}
        return is_infinite and not _contains_break(_block_stmts(stmt.child_by_field_name("body")))
    return False


def check(tree, source: bytes, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for func in _walk(tree.root_node):
        if func.type != "function_definition":
            continue
        if _is_void_function(func):
            continue

        if not _always_exits(func.child_by_field_name("body")):
            line, col = func.start_point[0] + 1, func.start_point[1]
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=line,
                    col=col,
                    rule_id=RULE_ID,
                    message=f"Function `{_function_name(func)}` may not return a value on all code paths",
                )
            )
    return diagnostics
