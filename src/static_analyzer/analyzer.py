"""Discovers Python source files and runs all enabled rules against them."""

from __future__ import annotations

import ast
import fnmatch
from pathlib import Path

from static_analyzer.config import Config
from static_analyzer.diagnostics import Diagnostic
from static_analyzer.rules import ALL_RULES


DEFAULT_EXCLUDE_DIRS = frozenset(
    {".venv", "venv", ".git", "__pycache__", "build", "dist", ".tox", ".mypy_cache", ".pytest_cache", "node_modules"}
)


def _is_excluded(path: Path, patterns: list[str]) -> bool:
    if DEFAULT_EXCLUDE_DIRS.intersection(path.parts) or any(part.endswith(".egg-info") for part in path.parts):
        return True
    posix = path.as_posix()
    return any(fnmatch.fnmatch(posix, pattern) or fnmatch.fnmatch(path.name, pattern) for pattern in patterns)


def iter_python_files(paths: list[Path], exclude: list[str]):
    for path in paths:
        if path.is_file():
            if path.suffix == ".py" and not _is_excluded(path, exclude):
                yield path
            continue
        for candidate in sorted(path.rglob("*.py")):
            if not _is_excluded(candidate, exclude):
                yield candidate


def analyze_file(path: Path, config: Config) -> list[Diagnostic]:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        return [Diagnostic(path=str(path), line=1, col=0, rule_id="SA000", message=f"Could not read file: {exc}")]

    try:
        tree = ast.parse(source, filename=str(path))
    except SyntaxError as exc:
        return [
            Diagnostic(
                path=str(path),
                line=exc.lineno or 1,
                col=exc.offset or 0,
                rule_id="SA000",
                message=f"Syntax error: {exc.msg}",
            )
        ]

    diagnostics: list[Diagnostic] = []
    for rule in ALL_RULES:
        if not config.is_enabled(rule.RULE_ID):
            continue
        diagnostics.extend(rule.check(tree, str(path), config))
    return diagnostics


def analyze_paths(paths: list[Path], config: Config) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for file_path in iter_python_files(paths, config.exclude):
        diagnostics.extend(analyze_file(file_path, config))
    return sorted(diagnostics)
