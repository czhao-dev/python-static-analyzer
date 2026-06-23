"""Registry of all available rules."""

from __future__ import annotations

from c_static_analyzer.rules import (
    complexity,
    missing_return,
    nesting,
    unreachable_code,
    unused_variables,
)

ALL_RULES = (
    complexity,
    unused_variables,
    nesting,
    missing_return,
    unreachable_code,
)

RULE_DESCRIPTIONS = {
    complexity.RULE_ID: "High cyclomatic complexity",
    unused_variables.RULE_ID: "Unused local variable",
    nesting.RULE_ID: "Deeply nested control flow",
    missing_return.RULE_ID: "Missing return path",
    unreachable_code.RULE_ID: "Unreachable code",
}
