"""SA002: Unused local variable."""

from __future__ import annotations

from c_static_analyzer.diagnostics import Diagnostic

RULE_ID = "SA002"


def _walk(node):
    yield node
    for child in node.children:
        yield from _walk(child)


def _base_identifier(declarator):
    while declarator is not None and declarator.type != "identifier":
        declarator = declarator.child_by_field_name("declarator")
    return declarator


def _declared_names(body):
    """First declaration site per local variable name, keyed by name."""
    declared: dict[str, object] = {}
    for node in _walk(body):
        if node.type != "declaration":
            continue
        for declarator in node.children_by_field_name("declarator"):
            name_node = _base_identifier(declarator)
            if name_node is None:
                continue
            name = name_node.text.decode("utf-8")
            if name.startswith("_"):
                continue
            declared.setdefault(name, name_node)
    return declared


def _is_plain_assignment_target(node) -> bool:
    parent = node.parent
    if parent is None or parent.type != "assignment_expression":
        return False
    operator = parent.child_by_field_name("operator")
    left = parent.child_by_field_name("left")
    return left is node and operator is not None and operator.text == b"="


def _collect_used(body, declared_site_ids: set[int]) -> set[str]:
    used: set[str] = set()
    for node in _walk(body):
        if node.type != "identifier":
            continue
        if node.id in declared_site_ids or _is_plain_assignment_target(node):
            continue
        used.add(node.text.decode("utf-8"))
    return used


def check(tree, source: bytes, path: str, config) -> list[Diagnostic]:
    diagnostics = []
    for func in _walk(tree.root_node):
        if func.type != "function_definition":
            continue
        body = func.child_by_field_name("body")
        if body is None:
            continue

        declared = _declared_names(body)
        declared_site_ids = {node.id for node in declared.values()}
        used = _collect_used(body, declared_site_ids)

        for name, name_node in declared.items():
            if name in used:
                continue
            line, col = name_node.start_point[0] + 1, name_node.start_point[1]
            diagnostics.append(
                Diagnostic(
                    path=path,
                    line=line,
                    col=col,
                    rule_id=RULE_ID,
                    message=f"Local variable `{name}` is assigned but never used",
                )
            )

    return diagnostics
