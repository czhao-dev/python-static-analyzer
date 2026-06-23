from helpers import parse

from c_static_analyzer.config import Config
from c_static_analyzer.rules import nesting


def test_shallow_nesting_is_not_flagged(config):
    tree, source = parse(
        """
int f(int x) {
    if (x) {
        return 1;
    }
    return 0;
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_flags_deep_nesting():
    cfg = Config(max_nesting=2)
    tree, source = parse(
        """
int f(int x) {
    if (x) {
        for (int i = 0; i < x; i++) {
            while (i > 0) {
                i--;
            }
        }
    }
    return 0;
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", cfg)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA003"


def test_elif_chain_does_not_count_as_nesting():
    cfg = Config(max_nesting=1)
    tree, source = parse(
        """
const char *f(int x) {
    if (x == 1) {
        return "one";
    } else if (x == 2) {
        return "two";
    } else if (x == 3) {
        return "three";
    } else {
        return "other";
    }
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", cfg)
    assert diagnostics == []


def test_else_with_nested_if_does_count():
    cfg = Config(max_nesting=1)
    tree, source = parse(
        """
int f(int x, int y) {
    if (x) {
        return 1;
    } else {
        if (y) {
            return 2;
        }
    }
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", cfg)
    assert len(diagnostics) == 1


def test_reports_only_once_per_function():
    cfg = Config(max_nesting=1)
    tree, source = parse(
        """
int f(int x) {
    if (x) {
        if (x) {
            if (x) {
                return 1;
            }
        }
    }
    return 0;
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", cfg)
    assert len(diagnostics) == 1


def test_switch_case_counts_as_nesting():
    cfg = Config(max_nesting=1)
    tree, source = parse(
        """
int f(int x) {
    switch (x) {
        case 1:
            if (x) {
                return 1;
            }
            return 0;
    }
    return 0;
}
"""
    )
    diagnostics = nesting.check(tree, source, "example.c", cfg)
    assert len(diagnostics) == 1
