# Python Static Analyzer

A lightweight Python static analyzer for finding common code quality, correctness, and maintainability issues before runtime.

It scans `.py` files with the `ast` module (no execution of your code), reports file-and-line diagnostics with stable rule IDs, and exits non-zero when it finds something — so it works as a local check or a CI gate.

## Implemented Checks

| Rule    | Description |
|---------|--------------|
| `SA001` | Mutable default argument, such as `def add_item(item, items=[])`. |
| `SA002` | Unused import. |
| `SA003` | Broad exception handler, such as `except Exception` or a bare `except:`. |
| `SA004` | Built-in name shadowed, such as `list`, `dict`, or `id`. |
| `SA005` | Function with high cyclomatic complexity. |
| `SA006` | Unused local variable. |
| `SA007` | Deeply nested control flow. |
| `SA008` | Missing return path in a function that appears to return a value. |
| `SA009` | Unreachable code after `return`, `raise`, `break`, or `continue`. |

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

The analyzer reports:

```text
example.py:1: SA001 Mutable default argument `values=[]`
example.py:5: SA003 Broad exception handler `except Exception`
```

## Installation

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

## Usage

```bash
static-analyzer scan path/to/project
# or, without installing a console script:
python -m static_analyzer scan path/to/project
```

Example output:

```text
src/app.py:12: SA001 Mutable default argument `items=[]`
src/app.py:34: SA002 Unused import `json`
src/service.py:48: SA003 Broad exception handler `except Exception`
```

The command exits `0` when no issues are found, `1` when diagnostics are reported, and `2` on a usage error (e.g. a path that doesn't exist) — making it suitable as a CI gate.

By default the scanner skips common non-project directories (`.venv`, `venv`, `.git`, `__pycache__`, `build`, `dist`, `*.egg-info`, `.tox`, caches, `node_modules`).

### CLI options

```text
static-analyzer scan [paths ...]
  --max-complexity N     Cyclomatic complexity threshold (default: 10)
  --max-nesting N        Control flow nesting depth threshold (default: 4)
  --select SA001,SA002   Only run these rule IDs (default: all rules)
  --exclude PATTERN       Glob pattern to exclude; repeatable
  --no-config             Ignore pyproject.toml configuration
```

## Configuration

Settings can be set on the command line or in `pyproject.toml`:

```toml
[tool.static-analyzer]
exclude = ["tests/fixtures/*"]
max_complexity = 10
max_nesting = 4
enabled_rules = ["SA001", "SA002", "SA004"]
```

`enabled_rules` defaults to an empty list, which means all rules are enabled. CLI flags override the values loaded from `pyproject.toml`.

## Development

Project structure:

```text
python-static-analyzer/
├── README.md
├── pyproject.toml
├── examples/
│   └── sample_issues.py
├── src/
│   └── static_analyzer/
│       ├── __init__.py
│       ├── __main__.py
│       ├── cli.py
│       ├── analyzer.py
│       ├── config.py
│       ├── diagnostics.py
│       └── rules/
│           ├── __init__.py
│           ├── mutable_defaults.py
│           ├── unused_imports.py
│           ├── unused_variables.py
│           ├── broad_exceptions.py
│           ├── shadowed_builtins.py
│           ├── complexity.py
│           ├── nesting.py
│           ├── missing_return.py
│           └── unreachable_code.py
└── tests/
    ├── test_analyzer.py
    ├── test_cli.py
    └── test_*.py  (one file per rule)
```

Development commands:

```bash
pip install -e ".[dev]"
pytest
static-analyzer scan examples/
```

## Design

The analyzer uses Python's built-in `ast` module:

1. Parse each `.py` file into an abstract syntax tree.
2. Run each enabled rule's `check(tree, path, config)` function over the tree.
3. Collect diagnostics with rule IDs, messages, file paths, and line numbers.
4. Sort and render results in a human-readable CLI format.
5. Exit with a non-zero status when findings are present, making the tool usable in CI.

Each rule lives in its own module under `src/static_analyzer/rules/` and exposes a `RULE_ID` and a `check()` function, so adding a new rule means adding one file and registering it in `rules/__init__.py`.

## Test Results

The project ships with 43 unit and end-to-end tests covering every rule plus the CLI:

```text
$ pytest -q
...........................................
43 passed in 0.03s
```

Running the analyzer on [examples/sample_issues.py](examples/sample_issues.py), a file written specifically to trigger every rule, confirms end-to-end behavior:

```text
$ static-analyzer scan examples/sample_issues.py
examples/sample_issues.py:1: SA002 Unused import `json`
examples/sample_issues.py:2: SA002 Unused import `os`
examples/sample_issues.py:5: SA001 Mutable default argument `values=[]`
examples/sample_issues.py:9: SA003 Broad exception handler `except Exception`
examples/sample_issues.py:13: SA004 Parameter `list` shadows a built-in name
examples/sample_issues.py:14: SA006 Local variable `unused` is assigned but never used
examples/sample_issues.py:18: SA008 Function `classify` may not return a value on all code paths
examples/sample_issues.py:23: SA008 Function `first_even` may not return a value on all code paths
examples/sample_issues.py:27: SA009 Unreachable code after `return`

9 issue(s) found.
```

Running the analyzer against its own source tree turns up only two honest complexity findings, on functions that are inherently branchy (a control-flow dispatcher and an import-resolution loop) — left as-is rather than artificially restructured to satisfy the linter:

```text
$ static-analyzer scan src
src/static_analyzer/rules/missing_return.py:38: SA005 Function `_stmt_always_exits` has cyclomatic complexity 16 (threshold 10)
src/static_analyzer/rules/unused_imports.py:48: SA005 Function `check` has cyclomatic complexity 12 (threshold 10)

2 issue(s) found.
```

## Roadmap

- [x] Add project packaging with `pyproject.toml`.
- [x] Implement the CLI entry point.
- [x] Implement AST parsing for Python files.
- [x] Add the first rule: mutable default arguments.
- [x] Add diagnostic formatting.
- [x] Add unit tests for each rule.
- [x] Add configuration support.
- [x] Add CI-friendly exit codes.
- [ ] Add JSON output for editor and automation integrations.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
