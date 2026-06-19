from static_analyzer.analyzer import iter_python_files
from static_analyzer.config import Config


def test_default_excludes_skip_venv_and_vcs_dirs(tmp_path):
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "app.py").write_text("x = 1\n")
    venv_pkg = tmp_path / ".venv" / "lib" / "somepkg"
    venv_pkg.mkdir(parents=True)
    (venv_pkg / "module.py").write_text("y = 2\n")

    found = sorted(p.name for p in iter_python_files([tmp_path], Config().exclude))

    assert found == ["app.py"]


def test_custom_exclude_pattern(tmp_path):
    (tmp_path / "keep.py").write_text("x = 1\n")
    (tmp_path / "generated.py").write_text("x = 1\n")

    found = sorted(p.name for p in iter_python_files([tmp_path], ["*generated*"]))

    assert found == ["keep.py"]
