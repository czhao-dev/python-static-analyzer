from tree_sitter import Language, Parser
import tree_sitter_c as tsc

C_LANGUAGE = Language(tsc.language())


def parse(source: str):
    """Parse C source text, returning (tree, source_bytes)."""
    source_bytes = source.encode("utf-8")
    parser = Parser(C_LANGUAGE)
    tree = parser.parse(source_bytes)
    return tree, source_bytes
