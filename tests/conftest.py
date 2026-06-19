import pytest

from static_analyzer.config import Config


@pytest.fixture
def config() -> Config:
    return Config()
