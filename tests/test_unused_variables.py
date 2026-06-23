from helpers import parse

from c_static_analyzer.rules import unused_variables


def test_flags_unused_local_variable(config):
    tree, source = parse(
        """
int compute(void) {
    int total = 0;
    int unused = 42;
    return total;
}
"""
    )
    diagnostics = unused_variables.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1
    assert "unused" in diagnostics[0].message
    assert diagnostics[0].rule_id == "SA002"


def test_ignores_used_variable(config):
    tree, source = parse(
        """
int compute(void) {
    int total = 0;
    for (int i = 0; i < 10; i++) {
        total += i;
    }
    return total;
}
"""
    )
    diagnostics = unused_variables.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_underscore_prefixed(config):
    tree, source = parse(
        """
int compute(void) {
    int _ignored = expensive_call();
    return 1;
}
"""
    )
    diagnostics = unused_variables.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_ignores_global_variable_mutation(config):
    tree, source = parse(
        """
int counter = 0;

void increment(void) {
    counter = counter + 1;
}
"""
    )
    diagnostics = unused_variables.check(tree, source, "example.c", config)
    assert diagnostics == []


def test_array_size_and_initializer_count_as_use(config):
    tree, source = parse(
        """
int compute(int n) {
    int size = n;
    int values[size];
    return values[0];
}
"""
    )
    diagnostics = unused_variables.check(tree, source, "example.c", config)
    assert diagnostics == []
