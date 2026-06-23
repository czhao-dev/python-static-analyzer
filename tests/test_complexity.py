from helpers import parse

from c_static_analyzer.config import Config
from c_static_analyzer.rules import complexity


def test_simple_function_is_not_flagged(config):
    tree, source = parse(
        """
int add(int a, int b) {
    return a + b;
}
"""
    )
    diagnostics = complexity.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_flags_high_complexity_function():
    cfg = Config(max_complexity=3)
    tree, source = parse(
        """
const char *classify(int x) {
    if (x > 0) {
        if (x > 10) {
            return "big";
        }
        return "small";
    } else if (x < 0) {
        return "negative";
    }
    return "zero";
}
"""
    )
    diagnostics = complexity.check(tree, source, "example.c", cfg)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA001"


def test_multiple_functions_scored_independently():
    cfg = Config(max_complexity=1)
    tree, source = parse(
        """
int outer(void) {
    return 1;
}

int inner(int x) {
    if (x) {
        return 1;
    }
    return 2;
}
"""
    )
    diagnostics = complexity.check(tree, source, "example.c", cfg)
    assert {d.message.split("`")[1] for d in diagnostics} == {"inner"}
