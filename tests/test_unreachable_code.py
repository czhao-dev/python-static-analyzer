from helpers import parse

from c_static_analyzer.rules import unreachable_code


def test_flags_code_after_return(config):
    tree, source = parse(
        """
int f(void) {
    return 1;
    printf("never runs");
}
"""
    )
    diagnostics = unreachable_code.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1
    assert diagnostics[0].rule_id == "SA005"
    assert diagnostics[0].line == 4


def test_flags_code_after_break_in_loop(config):
    tree, source = parse(
        """
void f(void) {
    for (int i = 0; i < 10; i++) {
        break;
        printf("%d", i);
    }
}
"""
    )
    diagnostics = unreachable_code.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1


def test_flags_code_after_return_in_case(config):
    tree, source = parse(
        """
int f(int x) {
    switch (x) {
        case 1:
            return 1;
            return 2;
    }
    return 0;
}
"""
    )
    diagnostics = unreachable_code.check(tree, source, "example.c", config)
    assert len(diagnostics) == 1


def test_ignores_reachable_code(config):
    tree, source = parse(
        """
int f(int x) {
    if (x) {
        return 1;
    }
    return 2;
}
"""
    )
    diagnostics = unreachable_code.check(tree, source, "example.c", config)
    assert diagnostics == []
