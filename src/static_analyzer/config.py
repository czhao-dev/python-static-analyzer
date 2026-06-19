"""Loading of optional [tool.static-analyzer] configuration from pyproject.toml."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover - Python < 3.11 fallback
    tomllib = None


@dataclass
class Config:
    exclude: list[str] = field(default_factory=list)
    max_complexity: int = 10
    max_nesting: int = 4
    enabled_rules: list[str] = field(default_factory=list)

    def is_enabled(self, rule_id: str) -> bool:
        return not self.enabled_rules or rule_id in self.enabled_rules


def load_config(start: Path) -> Config:
    """Load configuration from the nearest pyproject.toml above `start`."""
    config = Config()
    if tomllib is None:
        return config

    for directory in [start, *start.resolve().parents]:
        candidate = directory / "pyproject.toml"
        if candidate.is_file():
            try:
                data = tomllib.loads(candidate.read_text(encoding="utf-8"))
            except (OSError, tomllib.TOMLDecodeError):
                return config
            section = data.get("tool", {}).get("static-analyzer", {})
            config.exclude = list(section.get("exclude", config.exclude))
            config.max_complexity = int(section.get("max_complexity", config.max_complexity))
            config.max_nesting = int(section.get("max_nesting", config.max_nesting))
            config.enabled_rules = list(section.get("enabled_rules", config.enabled_rules))
            return config
    return config
