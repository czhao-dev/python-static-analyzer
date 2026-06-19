"""Diagnostic data structures shared by all rules."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, order=True)
class Diagnostic:
    """A single finding reported by a rule."""

    path: str
    line: int
    col: int
    rule_id: str
    message: str

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.rule_id} {self.message}"
