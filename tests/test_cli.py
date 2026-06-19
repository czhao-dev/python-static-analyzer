from static_analyzer.cli import main


def test_scan_clean_file_exits_zero(tmp_path, capsys):
    clean_file = tmp_path / "clean.py"
    clean_file.write_text("def add(a, b):\n    return a + b\n")

    exit_code = main(["scan", str(clean_file), "--no-config"])

    assert exit_code == 0
    out, _ = capsys.readouterr()
    assert out == ""


def test_scan_file_with_issues_exits_one(tmp_path, capsys):
    bad_file = tmp_path / "bad.py"
    bad_file.write_text(
        "def add_item(item, items=[]):\n"
        "    items.append(item)\n"
        "    return items\n"
    )

    exit_code = main(["scan", str(bad_file), "--no-config"])

    assert exit_code == 1
    out, _ = capsys.readouterr()
    assert "SA001" in out
    assert str(bad_file) in out


def test_select_filters_rules(tmp_path, capsys):
    bad_file = tmp_path / "bad.py"
    bad_file.write_text(
        "def add_item(item, items=[]):\n"
        "    items.append(item)\n"
        "    return items\n"
    )

    exit_code = main(["scan", str(bad_file), "--no-config", "--select", "SA005"])

    assert exit_code == 0
    out, _ = capsys.readouterr()
    assert out == ""


def test_missing_path_exits_two():
    exit_code = main(["scan", "/no/such/path.py", "--no-config"])
    assert exit_code == 2
