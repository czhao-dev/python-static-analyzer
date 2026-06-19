import ast


def parse(source: str) -> ast.Module:
    return ast.parse(source)
