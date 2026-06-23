"""Command-line entry point: `c-static-analyzer scan <paths>`."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from c_static_analyzer import __version__
from c_static_analyzer.analyzer import analyze_paths
from c_static_analyzer.config import Config, load_config


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="c-static-analyzer")
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")

    subparsers = parser.add_subparsers(dest="command", required=True)
    scan = subparsers.add_parser("scan", help="Scan C files for issues")
    scan.add_argument("paths", nargs="*", default=["."], help="Files or directories to scan")
    scan.add_argument("--max-complexity", type=int, default=None, help="Cyclomatic complexity threshold")
    scan.add_argument("--max-nesting", type=int, default=None, help="Control flow nesting depth threshold")
    scan.add_argument(
        "--select",
        metavar="SA001,SA002",
        default=None,
        help="Comma-separated list of rule IDs to enable (default: all)",
    )
    scan.add_argument(
        "--exclude",
        metavar="PATTERN",
        action="append",
        default=[],
        help="Glob pattern to exclude; can be passed multiple times",
    )
    scan.add_argument("--no-config", action="store_true", help="Ignore .c-static-analyzer.toml configuration")
    return parser


def _build_config(args: argparse.Namespace) -> Config:
    config = load_config(Path.cwd()) if not args.no_config else Config()
    if args.max_complexity is not None:
        config.max_complexity = args.max_complexity
    if args.max_nesting is not None:
        config.max_nesting = args.max_nesting
    if args.select is not None:
        config.enabled_rules = [rule_id.strip() for rule_id in args.select.split(",") if rule_id.strip()]
    config.exclude = [*config.exclude, *args.exclude]
    return config


def _first_missing_path(paths: list[Path]) -> Path | None:
    return next((path for path in paths if not path.exists()), None)


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command != "scan":
        parser.print_help()
        return 2

    paths = [Path(p) for p in args.paths]
    missing = _first_missing_path(paths)
    if missing is not None:
        print(f"error: path not found: {missing}", file=sys.stderr)
        return 2

    config = _build_config(args)
    diagnostics = analyze_paths(paths, config)
    for diagnostic in diagnostics:
        print(diagnostic)

    if diagnostics:
        print(f"\n{len(diagnostics)} issue(s) found.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
