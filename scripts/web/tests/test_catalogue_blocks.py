"""Tests of the entry-text blocks that scripts/web/generate_data.py catalogue writes.

The web page renders the research entry files from these blocks without
parsing Markdown (web phase 5.5). These tests check the conversion on small
inputs and on every entry file of the repository: no Markdown marker is left
in plain text, every citation becomes a citation span, every link to another
entry names an existing entry and heading, and every table row has as many
cells as its header. Standard library only:

    python -I -X utf8 scripts/web/tests/test_catalogue_blocks.py
"""

from __future__ import annotations

import importlib.util
import pathlib
import re
import sys
import unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("strato_generate_data", HERE.parent / "generate_data.py")
assert spec and spec.loader
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)

REL = "docs/research/x.md"


def _spans(blocks: list[dict]):
    """Every inline span of a block list, depth first."""

    def walk(spans):
        for s in spans:
            yield s
            if "c" in s:
                yield from walk(s["c"])

    for b in blocks:
        if b["t"] in ("p", "h"):
            yield from walk(b["c"])
        elif b["t"] in ("ul", "ol"):
            for it in b["items"]:
                yield from walk(it)
        elif b["t"] == "table":
            for c in b["head"]:
                yield from walk(c)
            for r in b["rows"]:
                for c in r:
                    yield from walk(c)


class Blocks(unittest.TestCase):
    def test_heading_paragraph_and_inline_spans(self) -> None:
        blocks = gen.markdown_blocks(
            "# Title\n\nSome **bold $a_s^*$ text** and `code`,\nwith $D_i$ [ref:x2020] and [an entry](y.md#why-it-matters).\n",
            REL,
        )
        self.assertEqual([b["t"] for b in blocks], ["h", "p"])
        self.assertEqual(blocks[0]["id"], "title")
        kinds = [s["t"] for s in blocks[1]["c"]]
        self.assertEqual(kinds, ["text", "strong", "text", "code", "text", "math", "text", "ref", "text", "link", "text"])
        strong = blocks[1]["c"][1]
        self.assertEqual([s["t"] for s in strong["c"]], ["text", "math", "text"])
        self.assertEqual(strong["c"][1]["v"], "a_s^*")
        link = blocks[1]["c"][9]
        self.assertEqual((link["entry"], link["anchor"]), ("docs/research/y.md", "why-it-matters"))
        self.assertEqual(blocks[1]["c"][7], {"t": "ref", "id": "x2020"})

    def test_display_maths_lists_and_tables(self) -> None:
        text = (
            "## Candidate governing equations (research concepts)\n\n"
            "$$\n\\frac{\\partial c}{\\partial t} = -k c\n$$\n\n"
            "$$ x = 1 $$\n\n"
            "- one\n  continued\n- two [ref:a]\n\n"
            "1. first\n2. second\n\n"
            "| A | B |\n|---|---|\n| $|x|$ | `a|b` |\n"
        )
        blocks = gen.markdown_blocks(text, REL)
        self.assertEqual([b["t"] for b in blocks], ["h", "math", "math", "ul", "ol", "table"])
        self.assertEqual(blocks[0]["id"], "candidate-governing-equations-research-concepts")
        self.assertEqual(blocks[1]["tex"], r"\frac{\partial c}{\partial t} = -k c")
        self.assertEqual(blocks[2]["tex"], "x = 1")
        self.assertEqual(blocks[3]["items"][0], [{"t": "text", "v": "one continued"}])
        self.assertEqual(blocks[4]["start"], 1)
        self.assertEqual(len(blocks[5]["head"]), 2)
        self.assertEqual(blocks[5]["rows"][0][0], [{"t": "math", "v": "|x|"}])
        self.assertEqual(blocks[5]["rows"][0][1], [{"t": "code", "v": "a|b"}])

    def test_repeated_headings_get_distinct_ids(self) -> None:
        blocks = gen.markdown_blocks("## Notes\n\n## Notes\n", REL)
        self.assertEqual([b["id"] for b in blocks], ["notes", "notes-1"])

    def test_list_cells_keep_a_semicolon_inside_parentheses(self) -> None:
        self.assertEqual(
            gen.split_list("a (missing);b (missing; only a range in x2012);c (registered: f.csv#A1)"),
            ["a (missing)", "b (missing; only a range in x2012)", "c (registered: f.csv#A1)"],
        )
        self.assertEqual(gen.split_list("x; y;;"), ["x", "y"])

    def test_reference_labels(self) -> None:
        rows = [
            {"reference_id": "a", "citation": "Smith, J. (2001). T.", "register_link": ""},
            {"reference_id": "b", "citation": "Smith, J., & Jones, K. (2002). T.", "register_link": ""},
            {"reference_id": "c", "citation": "Smith, J., Jones, K., & Lee, M. (2003). T.", "register_link": ""},
            {"reference_id": "d", "citation": "Agency. (2004). T.", "register_link": "Agency (2004)"},
            {"reference_id": "e", "citation": "Smith, J., Lee, K., & Jones, M. (2003). U.", "register_link": ""},
        ]
        labels = gen.reference_labels(rows)
        self.assertEqual(labels["a"], "Smith 2001")
        self.assertEqual(labels["b"], "Smith and Jones 2002")
        self.assertEqual(labels["c"], "Smith et al. 2003a")
        self.assertEqual(labels["e"], "Smith et al. 2003b")
        self.assertEqual(labels["d"], "Agency 2004")


class RepositoryEntries(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.doc = gen.build_catalogue(gen.ROOT)
        cls.entries = cls.doc["data"]["entries"]
        cls.refs = {r["reference_id"] for r in cls.doc["data"]["references"]}

    def test_every_entry_file_has_blocks_and_a_title(self) -> None:
        files = sorted(p.relative_to(gen.ROOT).as_posix() for p in (gen.ROOT / gen.ENTRY_DIR).glob("*.md"))
        self.assertEqual(sorted(self.entries), files)
        for rel, e in self.entries.items():
            with self.subTest(entry=rel):
                self.assertTrue(e["blocks"], rel)
                self.assertEqual(e["blocks"][0]["t"], "h")
                self.assertEqual(e["blocks"][0]["level"], 1)

    def test_no_markdown_marker_is_left_in_plain_text(self) -> None:
        for rel, e in self.entries.items():
            for s in _spans(e["blocks"]):
                if s["t"] == "text":
                    with self.subTest(entry=rel, text=s["v"][:60]):
                        for marker in ("$", "**", "[ref:", "](", "`"):
                            self.assertNotIn(marker, s["v"])

    def test_every_citation_is_a_citation_span_of_a_known_reference(self) -> None:
        for rel, e in self.entries.items():
            source = (gen.ROOT / rel).read_text(encoding="utf-8")
            in_text = re.findall(r"\[ref:([^\]\s]+)\]", source)
            spans = [s["id"] for s in _spans(e["blocks"]) if s["t"] == "ref"]
            with self.subTest(entry=rel):
                self.assertEqual(sorted(in_text), sorted(spans))
                self.assertLessEqual(set(spans), self.refs)

    def test_every_link_to_an_entry_resolves_to_an_entry_and_heading(self) -> None:
        anchors = {rel: {b["id"] for b in e["blocks"] if b["t"] == "h"} for rel, e in self.entries.items()}
        for rel, e in self.entries.items():
            for s in _spans(e["blocks"]):
                if s["t"] == "link" and "entry" in s:
                    with self.subTest(entry=rel, target=s["entry"]):
                        self.assertIn(s["entry"], self.entries)
                        if s["anchor"]:
                            self.assertIn(s["anchor"], anchors[s["entry"]])

    def test_every_display_equation_is_kept(self) -> None:
        for rel, e in self.entries.items():
            source = (gen.ROOT / rel).read_text(encoding="utf-8")
            n = len(re.findall(r"\$\$(.+?)\$\$", source, re.S))
            with self.subTest(entry=rel):
                self.assertEqual(sum(1 for b in e["blocks"] if b["t"] == "math"), n)

    def test_concept_equations_include_those_under_sub_headings(self) -> None:
        for rel, e in self.entries.items():
            blocks = e["blocks"]
            start = next((i for i, b in enumerate(blocks) if b["t"] == "h" and "candidate governing equations" in b["id"].replace("-", " ")), None)
            if start is None:
                continue
            level = blocks[start]["level"]
            section = []
            for b in blocks[start + 1:]:
                if b["t"] == "h" and b["level"] <= level:
                    break
                section.append(b)
            with self.subTest(entry=rel):
                self.assertEqual(e["concept_equations"], [b["tex"] for b in section if b["t"] == "math"])
                self.assertGreater(len(e["concept_equations"]), 0)

    def test_table_rows_match_their_header(self) -> None:
        for rel, e in self.entries.items():
            for b in e["blocks"]:
                if b["t"] == "table":
                    for r in b["rows"]:
                        with self.subTest(entry=rel):
                            self.assertEqual(len(r), len(b["head"]))

    def test_every_reference_has_a_label(self) -> None:
        labels = self.doc["data"]["reference_labels"]
        self.assertEqual(set(labels), self.refs)
        self.assertEqual(len(set(labels.values())), len(labels), "two references share one label")


if __name__ == "__main__":
    result = unittest.main(argv=[sys.argv[0], "-v"], exit=False).result
    sys.exit(0 if result.wasSuccessful() else 1)
