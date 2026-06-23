# C Static Analyzer

A lightweight C static analyzer for finding common code quality, correctness, and maintainability issues before runtime.

It parses `.c`/`.h` files with [tree-sitter](https://tree-sitter.github.io/tree-sitter/) and the `tree-sitter-c` grammar (no compilation or execution of your code), reports file-and-line diagnostics with stable rule IDs, and exits non-zero when it finds something — so it works as a local check or a CI gate.

> **Rust port available.** This project is being migrated to Rust for distribution as a single, dependency-free binary. The original Python implementation below remains the reference implementation during the transition; see [Rust Port](#rust-port) for the new implementation, its test results, and parity verification against this Python version.

## Implemented Checks

| Rule    | Description |
|---------|--------------|
| `SA001` | Function with high cyclomatic complexity. |
| `SA002` | Unused local variable. |
| `SA003` | Deeply nested control flow. |
| `SA004` | Missing return path in a non-void function. |
| `SA005` | Unreachable code after `return`, `break`, `continue`, or `goto`. |

## Example

Given this C file:

```c
const char *classify(int x) {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    }
}
```

The analyzer reports:

```text
example.c:1: SA004 Function `classify` may not return a value on all code paths
```

## Installation

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

## Usage

```bash
c-static-analyzer scan path/to/project
# or, without installing a console script:
python -m c_static_analyzer scan path/to/project
```

Example output:

```text
src/app.c:12: SA001 Function `parse_request` has cyclomatic complexity 14 (threshold 10)
src/app.c:34: SA002 Local variable `unused` is assigned but never used
src/util.c:48: SA004 Function `convert` may not return a value on all code paths
```

The command exits `0` when no issues are found, `1` when diagnostics are reported, and `2` on a usage error (e.g. a path that doesn't exist) — making it suitable as a CI gate.

By default the scanner skips common non-project directories (`.git`, `build`, `dist`, `cmake-build-debug`, `cmake-build-release`, `CMakeFiles`, `out`, `vendor`, `third_party`).

### CLI options

```text
c-static-analyzer scan [paths ...]
  --max-complexity N     Cyclomatic complexity threshold (default: 10)
  --max-nesting N        Control flow nesting depth threshold (default: 4)
  --select SA001,SA002   Only run these rule IDs (default: all rules)
  --exclude PATTERN       Glob pattern to exclude; repeatable
  --no-config             Ignore .c-static-analyzer.toml configuration
```

## Configuration

Settings can be set on the command line or in a `.c-static-analyzer.toml` file in (or above) the directory you're scanning from:

```toml
exclude = ["tests/fixtures/*"]
max_complexity = 10
max_nesting = 4
enabled_rules = ["SA001", "SA002", "SA004"]
```

`enabled_rules` defaults to an empty list, which means all rules are enabled. CLI flags override the values loaded from `.c-static-analyzer.toml`.

## Development

Project structure:

```text
c-static-analyzer/
├── README.md
├── pyproject.toml
├── examples/
│   └── sample_issues.c
├── src/
│   └── c_static_analyzer/
│       ├── __init__.py
│       ├── __main__.py
│       ├── cli.py
│       ├── analyzer.py
│       ├── config.py
│       ├── diagnostics.py
│       └── rules/
│           ├── __init__.py
│           ├── complexity.py
│           ├── unused_variables.py
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
c-static-analyzer scan examples/
```

## Design

The analyzer uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) with the `tree-sitter-c` grammar:

1. Parse each `.c`/`.h` file into a concrete syntax tree.
2. Run each enabled rule's `check(tree, source, path, config)` function over the tree.
3. Collect diagnostics with rule IDs, messages, file paths, and line numbers.
4. Sort and render results in a human-readable CLI format.
5. Exit with a non-zero status when findings are present, making the tool usable in CI.

Each rule lives in its own module under `src/c_static_analyzer/rules/` and exposes a `RULE_ID` and a `check()` function, so adding a new rule means adding one file and registering it in `rules/__init__.py`.

## Test Results

The project ships with 32 unit and end-to-end tests covering every rule plus the CLI:

```text
$ pytest -q
................................
32 passed in 0.03s
```

Running the analyzer on [examples/sample_issues.c](examples/sample_issues.c), a file written specifically to trigger every rule, confirms end-to-end behavior:

```text
$ c-static-analyzer scan examples/sample_issues.c
examples/sample_issues.c:3: SA001 Function `complex_calc` has cyclomatic complexity 12 (threshold 10)
examples/sample_issues.c:18: SA004 Function `classify` may not return a value on all code paths
examples/sample_issues.c:31: SA003 Control flow nested 5 levels deep (threshold 4)
examples/sample_issues.c:41: SA002 Local variable `unused` is assigned but never used
examples/sample_issues.c:45: SA005 Unreachable code after `return`

5 issue(s) found.
```

## Rust Port

A Rust port lives in [`rust/`](rust/) and is a behavioral drop-in replacement for the Python CLI above: same 5 rules, same rule IDs and diagnostic messages, same CLI flags, same `.c-static-analyzer.toml` config semantics, same default excludes, and the same sorted, line-oriented output format and exit codes (`0`/`1`/`2`).

It uses the [`tree-sitter`](https://crates.io/crates/tree-sitter) and [`tree-sitter-c`](https://crates.io/crates/tree-sitter-c) crates — the same parser and grammar as the Python implementation, just through their native Rust bindings.

### Building and running

```bash
cd rust
cargo build --release
./target/release/c-static-analyzer scan path/to/project
```

The CLI surface (subcommand, flags, exit codes) is identical to the Python version documented above.

### Test results

44 tests pass — 36 unit tests (rule logic, config loading, fnmatch, file discovery) plus 8 integration tests (CLI behavior and a byte-for-byte golden-output comparison against the Python implementation):

```text
$ cargo test
...
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/analyzer.rs)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/cli.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/golden.rs)
```

### Parity verification

Beyond the unit/integration tests, the Rust binary's output was diffed directly against the Python CLI (`c-static-analyzer scan <dir> --no-config`) on `examples/sample_issues.c` and against the Python source tree (`src/`) — both byte-for-byte identical on stdout and stderr:

```text
$ diff <(python -m c_static_analyzer scan examples/sample_issues.c --no-config) \
       <(./rust/target/release/c-static-analyzer scan examples/sample_issues.c --no-config)
(no output — identical)
```

Since standard C has no nested function definitions, the Rust port's rule logic is a much more direct translation of the Python implementation than the earlier Python-analyzing version was: each rule walks `tree_sitter::Node`s with `child_by_field_name` instead of matching on `ast` node variants, but the underlying algorithms (complexity scoring, break/exit-path analysis, nesting depth tracking) are unchanged.

## Roadmap

- [x] Add project packaging with `pyproject.toml`.
- [x] Implement the CLI entry point.
- [x] Implement tree-sitter parsing for C files.
- [x] Add diagnostic formatting.
- [x] Add unit tests for each rule.
- [x] Add configuration support.
- [x] Add CI-friendly exit codes.
- [ ] Add JSON output for editor and automation integrations.
- [x] Port the analyzer to Rust as a dependency-free single binary (see [Rust Port](#rust-port)).
- [ ] Cut over to the Rust implementation as canonical and retire the Python source, once the port has had time to bake.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
