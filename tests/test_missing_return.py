from helpers import parse

from c_static_analyzer.rules import missing_return


def test_flags_missing_else_branch(config):
    tree, source = parse(
        """
const char *classify(int x) {
    if (x > 0) {
        return "positive";
    }
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA004"


def test_ignores_complete_if_else(config):
    tree, source = parse(
        """
const char *classify(int x) {
    if (x > 0) {
        return "positive";
    } else {
        return "non-positive";
    }
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_void_function(config):
    tree, source = parse(
        """
void log_message(const char *message) {
    printf("%s", message);
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_exhaustive_if_elif_else_chain(config):
    tree, source = parse(
        """
int parse_value(const char *value) {
    if (value == 0) {
        return -1;
    } else if (*value == '\\0') {
        return 0;
    } else {
        return atoi(value);
    }
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_infinite_while_loop_without_break(config):
    tree, source = parse(
        """
const char *serve(void) {
    while (1) {
        if (should_stop()) {
            return "done";
        }
        handle();
    }
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_infinite_do_while_loop_without_break(config):
    tree, source = parse(
        """
int serve_loop(void) {
    do {
        if (should_stop()) {
            return 1;
        }
    } while (1);
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_flags_while_loop_with_break(config):
    tree, source = parse(
        """
int find_first(int n) {
    while (1) {
        if (n > 0) {
            break;
        }
    }
}
"""
    )
    diagnostics = missing_return.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1
