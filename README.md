# Python Static Analyzer

A lightweight Python static analyzer for finding common code quality, correctness, and maintainability issues before runtime.

This project is intended to start small and grow into a configurable command-line tool that can scan Python projects, report actionable findings, and support custom rules.

## Goals

- Detect common Python mistakes using static analysis.
- Provide clear, file-and-line based diagnostics.
- Keep the rule system simple enough to extend.
- Offer useful defaults without requiring heavy configuration.
- Make the tool fast enough for local development and CI checks.

## Planned Checks

Initial rules may include:

- Mutable default arguments, such as `def add_item(item, items=[])`.
- Unused imports.
- Unused local variables.
- Broad exception handlers, such as `except Exception`.
- Shadowing built-in names, such as `list`, `dict`, or `id`.
- Functions with high cyclomatic complexity.
- Deeply nested control flow.
- Missing return paths in functions that appear to return values.
- Unreachable code after `return`, `raise`, `break`, or `continue`.

## Example

Given this Python file:

```python
def collect(value, values=[]):
    try:
        values.append(value)
        return values
    except Exception:
        return []
```

The analyzer could report:

```text
example.py:1: Mutable default argument `values=[]`
example.py:5: Avoid broad exception handler `except Exception`
```

## Proposed Usage

```bash
static-analyzer scan path/to/project
```

Example output:

```text
src/app.py:12: SA001 Mutable default argument
src/app.py:34: SA002 Unused import `json`
src/service.py:48: SA003 Broad exception handler
```

## Installation

This project is still under development. Once packaging is added, the expected local setup will be:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .
```

## Development

Suggested project structure:

```text
static-analyzer/
├── README.md
├── pyproject.toml
├── src/
│   └── static_analyzer/
│       ├── __init__.py
│       ├── cli.py
│       ├── analyzer.py
│       ├── diagnostics.py
│       └── rules/
│           ├── __init__.py
│           ├── mutable_defaults.py
│           ├── unused_imports.py
│           └── broad_exceptions.py
└── tests/
    ├── test_mutable_defaults.py
    ├── test_unused_imports.py
    └── test_broad_exceptions.py
```

Useful development commands, once implemented:

```bash
python -m static_analyzer scan examples/
pytest
ruff check .
```

## Design Sketch

The analyzer will likely use Python's built-in `ast` module:

1. Parse each `.py` file into an abstract syntax tree.
2. Run a set of rule visitors over the tree.
3. Collect diagnostics with rule IDs, messages, file paths, and line numbers.
4. Render results in a human-readable CLI format.
5. Exit with a non-zero status when findings are present, making the tool usable in CI.

## Rule Format

Each rule should produce diagnostics with a stable rule ID:

```text
SA001 Mutable default argument
SA002 Unused import
SA003 Broad exception handler
SA004 Built-in name shadowed
SA005 High cyclomatic complexity
```

Future versions may support configuration through a file such as:

```toml
[tool.static-analyzer]
exclude = ["tests/fixtures"]
max_complexity = 10
enabled_rules = ["SA001", "SA002", "SA003"]
```

## Roadmap

- [ ] Add project packaging with `pyproject.toml`.
- [ ] Implement the CLI entry point.
- [ ] Implement AST parsing for Python files.
- [ ] Add the first rule: mutable default arguments.
- [ ] Add diagnostic formatting.
- [ ] Add unit tests for each rule.
- [ ] Add configuration support.
- [ ] Add CI-friendly exit codes.
- [ ] Add JSON output for editor and automation integrations.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
