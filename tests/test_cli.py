from c_static_analyzer.cli import main


def test_scan_clean_file_exits_zero(tmp_path, capsys):
    clean_file = tmp_path / "clean.c"
    clean_file.write_text("int add(int a, int b) {\n    return a + b;\n}\n")

    exit_code = main(["scan", str(clean_file), "--no-config"])

    assert exit_code == 0
    out, _ = capsys.readouterr()
    assert out == ""


def test_scan_file_with_issues_exits_one(tmp_path, capsys):
    bad_file = tmp_path / "bad.c"
    bad_file.write_text(
        "int classify(int x) {\n"
        "    if (x > 0) {\n"
        "        return 1;\n"
        "    }\n"
        "}\n"
    )

    exit_code = main(["scan", str(bad_file), "--no-config"])

    assert exit_code == 1
    out, _ = capsys.readouterr()
    assert "SA004" in out
    assert str(bad_file) in out


def test_select_filters_rules(tmp_path, capsys):
    bad_file = tmp_path / "bad.c"
    bad_file.write_text(
        "int classify(int x) {\n"
        "    if (x > 0) {\n"
        "        return 1;\n"
        "    }\n"
        "}\n"
    )

    exit_code = main(["scan", str(bad_file), "--no-config", "--select", "SA001"])

    assert exit_code == 0
    out, _ = capsys.readouterr()
    assert out == ""


def test_missing_path_exits_two():
    exit_code = main(["scan", "/no/such/path.c", "--no-config"])
    assert exit_code == 2
