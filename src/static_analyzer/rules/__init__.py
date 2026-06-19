"""Registry of all available rules."""

from __future__ import annotations

from static_analyzer.rules import (
    broad_exceptions,
    complexity,
    missing_return,
    mutable_defaults,
    nesting,
    shadowed_builtins,
    unreachable_code,
    unused_imports,
    unused_variables,
)

ALL_RULES = (
    mutable_defaults,
    unused_imports,
    unused_variables,
    broad_exceptions,
    shadowed_builtins,
    complexity,
    nesting,
    missing_return,
    unreachable_code,
)

RULE_DESCRIPTIONS = {
    mutable_defaults.RULE_ID: "Mutable default argument",
    unused_imports.RULE_ID: "Unused import",
    unused_variables.RULE_ID: "Unused local variable",
    broad_exceptions.RULE_ID: "Broad exception handler",
    shadowed_builtins.RULE_ID: "Built-in name shadowed",
    complexity.RULE_ID: "High cyclomatic complexity",
    nesting.RULE_ID: "Deeply nested control flow",
    missing_return.RULE_ID: "Missing return path",
    unreachable_code.RULE_ID: "Unreachable code",
}
