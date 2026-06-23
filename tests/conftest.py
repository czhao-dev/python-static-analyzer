import pytest

from c_static_analyzer.config import Config


@pytest.fixture
def config() -> Config:
    return Config()
