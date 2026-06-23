"""Discovers C source files and runs all enabled rules against them."""

from __future__ import annotations

import fnmatch
from pathlib import Path

from tree_sitter import Language, Parser
import tree_sitter_c as tsc

from c_static_analyzer.config import Config
from c_static_analyzer.diagnostics import Diagnostic
from c_static_analyzer.rules import ALL_RULES

C_LANGUAGE = Language(tsc.language())

DEFAULT_EXCLUDE_DIRS = frozenset(
    {".git", "build", "dist", "cmake-build-debug", "cmake-build-release", "CMakeFiles", "out", "vendor", "third_party"}
)

_EXTENSIONS = (".c", ".h")


def _is_excluded(path: Path, patterns: list[str]) -> bool:
    if DEFAULT_EXCLUDE_DIRS.intersection(path.parts):
        return True
    posix = path.as_posix()
    return any(fnmatch.fnmatch(posix, pattern) or fnmatch.fnmatch(path.name, pattern) for pattern in patterns)


def iter_c_files(paths: list[Path], exclude: list[str]):
    for path in paths:
        if path.is_file():
            if path.suffix in _EXTENSIONS and not _is_excluded(path, exclude):
                yield path
            continue
        candidates = [c for ext in _EXTENSIONS for c in path.rglob(f"*{ext}")]
        for candidate in sorted(candidates):
            if not _is_excluded(candidate, exclude):
                yield candidate


def analyze_file(path: Path, config: Config) -> list[Diagnostic]:
    try:
        source = path.read_bytes()
    except OSError as exc:
        return [Diagnostic(path=str(path), line=1, col=0, rule_id="SA000", message=f"Could not read file: {exc}")]

    parser = Parser(C_LANGUAGE)
    tree = parser.parse(source)

    diagnostics: list[Diagnostic] = []
    for rule in ALL_RULES:
        if not config.is_enabled(rule.RULE_ID):
            continue
        diagnostics.extend(rule.check(tree, source, str(path), config))
    return diagnostics


def analyze_paths(paths: list[Path], config: Config) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for file_path in iter_c_files(paths, config.exclude):
        diagnostics.extend(analyze_file(file_path, config))
    return sorted(diagnostics)
