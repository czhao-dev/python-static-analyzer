from c_static_analyzer.analyzer import iter_c_files
from c_static_analyzer.config import Config


def test_default_excludes_skip_build_and_vcs_dirs(tmp_path):
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "app.c").write_text("int main(void) { return 0; }\n")
    build_dir = tmp_path / "build" / "obj"
    build_dir.mkdir(parents=True)
    (build_dir / "generated.c").write_text("int x;\n")

    found = sorted(p.name for p in iter_c_files([tmp_path], Config().exclude))

    assert found == ["app.c"]


def test_custom_exclude_pattern(tmp_path):
    (tmp_path / "keep.c").write_text("int x;\n")
    (tmp_path / "generated.c").write_text("int x;\n")

    found = sorted(p.name for p in iter_c_files([tmp_path], ["*generated*"]))

    assert found == ["keep.c"]


def test_discovers_header_files(tmp_path):
    (tmp_path / "lib.h").write_text("int add(int a, int b);\n")
    (tmp_path / "lib.c").write_text("int add(int a, int b) { return a + b; }\n")

    found = sorted(p.name for p in iter_c_files([tmp_path], []))

    assert found == ["lib.c", "lib.h"]
