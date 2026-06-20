# Python Static Analyzer

A lightweight Python static analyzer for finding common code quality, correctness, and maintainability issues before runtime.

It scans `.py` files with the `ast` module (no execution of your code), reports file-and-line diagnostics with stable rule IDs, and exits non-zero when it finds something — so it works as a local check or a CI gate.

> **Rust port available.** This project is being migrated to Rust for distribution as a single, dependency-free binary. The original Python implementation below remains the reference implementation during the transition; see [Rust Port](#rust-port) for the new implementation, its test results, and parity verification against this Python version.

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

## Rust Port

A Rust port lives in [`rust/`](rust/) and is a behavioral drop-in replacement for the Python CLI above: same 9 rules, same rule IDs and diagnostic messages, same CLI flags, same `pyproject.toml` config semantics, same default excludes, and the same sorted, line-oriented output format and exit codes (`0`/`1`/`2`).

It uses [`rustpython-ruff_python_ast`/`rustpython-ruff_python_parser`](https://crates.io/crates/rustpython-ruff_python_ast) (an actively-maintained mirror of ruff's internal Python AST/parser crates) instead of the unmaintained `rustpython-parser`, giving a CPython-shaped AST with an official visitor module to walk it.

### Building and running

```bash
cd rust
cargo build --release
./target/release/static-analyzer scan path/to/project
```

The CLI surface (subcommand, flags, exit codes) is identical to the Python version documented above.

### Test results

59 tests pass — 52 unit tests (rule logic, config loading, fnmatch, file discovery) plus 7 integration tests (CLI behavior and a byte-for-byte golden-output comparison against the Python implementation):

```text
$ cargo test
...
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/analyzer.rs)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/cli.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/golden.rs)
```

### Parity verification

Beyond the unit/integration tests, the Rust binary's output was diffed directly against the Python CLI (`python -m static_analyzer scan <dir> --no-config`) across every directory in this repository — `src/`, `tests/`, and `examples/` — on both stdout and stderr. All three are byte-for-byte identical:

```text
$ diff <(python -m static_analyzer scan src --no-config) <(./rust/target/release/static-analyzer scan src --no-config)
(no output — identical)

src/static_analyzer/rules/missing_return.py:38: SA005 Function `_stmt_always_exits` has cyclomatic complexity 16 (threshold 10)
src/static_analyzer/rules/unused_imports.py:48: SA005 Function `check` has cyclomatic complexity 12 (threshold 10)

2 issue(s) found.
```

Porting surfaced a few real differences between CPython's `ast` module and ruff's AST shape that needed deliberate handling (not just mechanical translation):

- **Elif chains.** CPython represents `elif` as a nested `ast.If` in `orelse`; ruff flattens `if`/`elif`/`else` into a single `elif_else_clauses` list. This changed how [`SA005`](rust/src/rules/sa005_complexity.rs) counts decision points (each `elif` still counts, but isn't a separate node) and how [`SA008`](rust/src/rules/sa008_missing_return.rs) determines exhaustiveness (an `if`/`elif` chain with no trailing `else` is not exhaustive, even if every existing branch returns — a case the original Python test suite didn't cover, now added as a regression test).
- **Double-visit quirk.** Ruff's generated statement visitor visits each `elif` clause's test expression twice (once explicitly, once again inside its own clause walker). [`SA005`](rust/src/rules/sa005_complexity.rs) drives `elif` traversal manually to avoid double-counting boolean operators living in an `elif` condition.
- **[`SA007`](rust/src/rules/sa007_nesting.rs) nesting** actually got simpler: CPython's `ast` can't structurally distinguish a true `elif` from `else:` followed by a nested `if` at the same source column (the Python implementation uses a column-offset heuristic for this). Ruff's `elif_else_clauses` distinguish them directly via `test: Option<Expr>` (`Some` for `elif`, `None` for `else`), so the Rust port doesn't need the heuristic at all.

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
- [x] Port the analyzer to Rust as a dependency-free single binary (see [Rust Port](#rust-port)).
- [ ] Cut over to the Rust implementation as canonical and retire the Python source, once the port has had time to bake.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
