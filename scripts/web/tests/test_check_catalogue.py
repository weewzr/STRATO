"""Tests of scripts/web/check_catalogue.py against its fixture folders.

prompt/TESTING_AND_VALIDATION.md Section 3.1: the check passes on the
repository and on the good fixture, and each bad fixture fails with the
matching FAIL line. A bad fixture is a folder with a case.json that names the
edits applied to a copy of the good fixture, the rules that must fail and the
messages they must print. Standard library only:

    python -I -X utf8 scripts/web/tests/test_check_catalogue.py
"""

from __future__ import annotations

import importlib.util
import json
import pathlib
import shutil
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
WEB = HERE.parent
FIXTURES = HERE / "fixtures" / "catalogue"


def _load(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


check = _load("strato_check_catalogue", WEB / "check_catalogue.py")
gen = check.gen


def _failed_rules(rep) -> set[int]:
    return {int(line.split()[1]) for line in rep.lines if line.startswith("FAIL ")}


def _apply(root: pathlib.Path, patches: list[dict]) -> None:
    for p in patches:
        path = root / p["file"]
        text = path.read_text(encoding="utf-8")
        if "append" in p:
            text += p["append"]
        else:
            assert p["find"] in text, f"fixture edit does not apply: {p['find']!r} not in {p['file']}"
            text = text.replace(p["find"], p["replace"], 1)
        path.write_text(text, encoding="utf-8", newline="\n")


class CheckCatalogue(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = pathlib.Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _copy_good(self) -> pathlib.Path:
        root = self.tmp / "root"
        shutil.copytree(FIXTURES / "good", root)
        return root

    def _generate(self, root: pathlib.Path) -> None:
        target = root / gen.CATALOGUE_JSON
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(gen.catalogue_text(gen.build_catalogue(root)), encoding="utf-8", newline="\n")

    def test_repository_passes(self) -> None:
        rep = check.check(gen.ROOT)
        self.assertFalse(rep.failed, "\n".join(rep.lines))

    def test_good_fixture_passes_every_rule_without_skipping(self) -> None:
        root = self._copy_good()
        self._generate(root)
        rep = check.check(root)
        self.assertFalse(rep.failed, "\n".join(rep.lines))
        self.assertFalse([l for l in rep.lines if l.startswith("SKIP")], "\n".join(rep.lines))
        self.assertEqual({int(l.split()[1]) for l in rep.lines if l.startswith("PASS")}, set(range(1, 10)))

    def test_good_fixture_extracts_the_research_concept_equation(self) -> None:
        root = self._copy_good()
        doc = gen.build_catalogue(root)
        entry = doc["data"]["entries"]["docs/research/fixture_pending.md"]
        self.assertEqual(entry["concept_equations"], [r"\frac{\partial c}{\partial t} = -k\,c"])

    def test_missing_generated_file_fails_rule_9(self) -> None:
        root = self._copy_good()
        rep = check.check(root)
        self.assertEqual(_failed_rules(rep), {9})

    def test_crlf_line_ends_fail_rule_1(self) -> None:
        # Made here, not stored as a fixture: .gitattributes normalises stored line ends to LF.
        root = self._copy_good()
        path = root / "data" / "research" / "references.csv"
        path.write_bytes(path.read_bytes().replace(b"\n", b"\r\n"))
        self._generate(root)
        rep = check.check(root)
        self.assertIn(1, _failed_rules(rep))
        self.assertTrue(any("CR line ends" in l for l in rep.lines))

    def test_overview_pages_need_no_row(self) -> None:
        root = self._copy_good()
        for rel in check.OVERVIEW_PAGES:
            (root / rel).write_text("# Overview\n\nFixture overview page [ref:fixture2025].\n", encoding="utf-8", newline="\n")
        self._generate(root)
        rep = check.check(root)
        self.assertFalse(rep.failed, "\n".join(rep.lines))

    def test_other_unlisted_page_fails_rule_4(self) -> None:
        root = self._copy_good()
        (root / "docs" / "research" / "regime-d.md").write_text("# Stray\n", encoding="utf-8", newline="\n")
        self._generate(root)
        rep = check.check(root)
        self.assertEqual(_failed_rules(rep), {4})
        self.assertTrue(any("docs/research/regime-d.md is referenced by no catalogue row" in l for l in rep.lines))

    def test_overview_page_reference_ids_are_checked(self) -> None:
        root = self._copy_good()
        (root / "docs" / "research" / "roadmap.md").write_text("# Roadmap\n\n[ref:nosuchref]\n", encoding="utf-8", newline="\n")
        self._generate(root)
        rep = check.check(root)
        self.assertEqual(_failed_rules(rep), {3})

    def test_each_bad_fixture_fails_with_its_line(self) -> None:
        cases = sorted(p for p in FIXTURES.iterdir() if p.is_dir() and p.name.startswith("bad_"))
        self.assertGreaterEqual(len(cases), 14)
        for case in cases:
            spec = json.loads((case / "case.json").read_text(encoding="utf-8"))
            with self.subTest(case=case.name):
                if (self.tmp / "root").exists():
                    shutil.rmtree(self.tmp / "root")
                root = self._copy_good()
                if spec.get("regenerate_after_patch", True):
                    _apply(root, spec["patches"])
                    self._generate(root)
                else:
                    self._generate(root)
                    _apply(root, spec["patches"])
                rep = check.check(root)
                failed = _failed_rules(rep)
                expected = set(spec["fail_rules"])
                text = "\n".join(rep.lines)
                if spec.get("allow_other_failures"):
                    self.assertTrue(expected <= failed, text)
                else:
                    self.assertEqual(failed, expected, text)
                for msg in spec["messages"]:
                    self.assertTrue(any(l.startswith("FAIL ") and msg in l for l in rep.lines), f"{msg!r} not printed:\n{text}")


if __name__ == "__main__":
    result = unittest.main(argv=[sys.argv[0], "-v"], exit=False).result
    sys.exit(0 if result.wasSuccessful() else 1)
