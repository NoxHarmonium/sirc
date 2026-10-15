#!/usr/bin/env python3
"""
SIRC-1 reference manual QA.

Builds the PDF, then verifies every code listing the manual claims is SIRC-1
assembly actually assembles with the real toolchain, and checks the build log
for unresolved cross-references. Everything runs locally; nothing here talks
to the network or to GitHub.

Usage:
    python3 qa.py [--skip-build] [--render PAGES]

Exit status is non-zero if the build fails, a reference is left undefined, or
any verified listing fails to assemble.

What "verified" means: a listing is only fed to the real assembler if it is
either (a) an external .sasm file pulled in with \\lstinputlisting (the
worked examples in Appendix D), (b) an inline \\begin{lstlisting} block inside
a \\begin{example}...\\end{example} float (the Usage Examples scattered
through the instruction chapters), or (c) an inline block explicitly opted in
with a "% qa:assemble" comment on the line before \\begin{lstlisting}. Bare
inline listings -- Syntax/Operation blocks using placeholder operands like
"#opcode" or pseudocode like "if condition_true then" -- are not real SIRC-1
programs and are skipped by default; that's a deliberate default, not a gap,
so the script reports how many it skipped rather than silently ignoring them.
"""

import argparse
import re
import subprocess
import sys
import tempfile
from pathlib import Path

DOCS_DIR = Path(__file__).resolve().parent
REPO_ROOT = DOCS_DIR.parents[1]
CHAPTERS_DIR = DOCS_DIR / "chapters"
VM_DIR = REPO_ROOT / "sirc-vm"
ASSEMBLER_BIN = VM_DIR / "target" / "debug" / "assembler"
LINKER_BIN = VM_DIR / "target" / "debug" / "linker"

SKIP_MARKER_RE = re.compile(r"^\s*%\s*qa:assemble\s*$")

LSTINPUTLISTING_RE = re.compile(r"\\lstinputlisting(?:\[[^\]]*\])?\{([^}]+)\}")
LSTLISTING_RE = re.compile(
    r"\\begin\{lstlisting\}(?:\[[^\]]*\])?\n(.*?)\\end\{lstlisting\}", re.DOTALL
)
EXAMPLE_FLOAT_RE = re.compile(r"\\begin\{example\}.*?\\end\{example\}", re.DOTALL)

UNDEFINED_REF_RE = re.compile(r"^LaTeX Warning: Reference `([^']+)' on page (\d+) undefined")
UNDEFINED_CITE_RE = re.compile(r"^LaTeX Warning: Citation `([^']+)' .* undefined")


class Failure:
    def __init__(self, location, message, detail=""):
        self.location = location
        self.message = message
        self.detail = detail

    def __str__(self):
        s = f"  {self.location}: {self.message}"
        if self.detail:
            indented = "\n".join("      " + line for line in self.detail.strip().splitlines())
            s += "\n" + indented
        return s


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def section(title):
    print(f"\n== {title} ==")


def check_format():
    section("Format check")
    result = run(["make", "format-check"], cwd=DOCS_DIR)
    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr)
        return [Failure("Makefile", "make format-check failed -- run `make format`")]
    print("OK: formatting matches latexindent's expected style.")
    return []


def build_pdf():
    section("Building PDF (make all)")
    result = run(["make", "all"], cwd=DOCS_DIR)
    log_tail = (result.stdout + result.stderr)[-4000:]
    if result.returncode != 0:
        print(log_tail)
        return False, [Failure("make all", "PDF build failed", log_tail)]
    print("OK: PDF built.")
    return True, []


def check_references():
    section("Cross-reference check")
    log_path = DOCS_DIR / "main.log"
    if not log_path.exists():
        return [Failure("main.log", "no build log found -- did the build run?")]
    text = log_path.read_text(errors="replace")
    failures = []
    for line in text.splitlines():
        m = UNDEFINED_REF_RE.match(line)
        if m:
            failures.append(
                Failure(f"page {m.group(2)}", f"undefined reference to `{m.group(1)}'")
            )
        m = UNDEFINED_CITE_RE.match(line)
        if m:
            failures.append(Failure("citation", f"undefined citation `{m.group(1)}'"))
    if failures:
        for f in failures:
            print(f)
    else:
        print("OK: no undefined references or citations after a full 3-pass build.")
    return failures


def ensure_toolchain_built():
    section("Building assembler/linker")
    result = run(["cargo", "build", "--bin", "assembler", "--bin", "linker", "--quiet"], cwd=VM_DIR)
    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr)
        return False
    print("OK: assembler and linker built.")
    return True


def assemble(src_path, tmp_dir, do_link):
    obj_path = tmp_dir / "listing.o"
    result = run([str(ASSEMBLER_BIN), "-i", str(src_path), "-o", str(obj_path)])
    if result.returncode != 0:
        return False, (result.stdout + result.stderr).strip()
    if not do_link:
        return True, ""
    bin_path = tmp_dir / "listing.bin"
    result = run(
        [str(LINKER_BIN), "--segment-offset", "0", "--output-file", str(bin_path), str(obj_path)]
    )
    if result.returncode != 0:
        return False, (result.stdout + result.stderr).strip()
    return True, ""


def find_example_spans(text):
    return [m.span() for m in EXAMPLE_FLOAT_RE.finditer(text)]


def line_of(text, offset):
    return text.count("\n", 0, offset) + 1


def preceding_nonblank_line(text, offset):
    line_start = text.rfind("\n", 0, offset)
    while line_start > 0:
        prev_start = text.rfind("\n", 0, line_start)
        candidate = text[prev_start + 1 : line_start]
        if candidate.strip():
            return candidate
        line_start = prev_start
    return ""


def check_external_listings():
    # Each \lstinputlisting'd program lives in its own examples/<name>/
    # directory, which already has a Makefile with a `check` target: build,
    # run under the VM, and diff the result against a checked-in expected
    # fixture (register dump, hex dump, or captured output). That's a much
    # stronger test than "does it assemble" and it's the same toolchain used
    # to actually validate these examples, so reuse it rather than
    # reimplementing assemble/link/run here. Multi-file examples (e.g.
    # hardware-exception.sasm + data.sasm) are handled correctly for free,
    # since `make check` builds whatever the example's own Makefile says it
    # needs.
    section("Verifying worked-example programs (make check)")
    failures = []
    example_dirs = {}  # examples/<name> -> representative (tex_file, rel_path) for reporting
    for tex_file in sorted(CHAPTERS_DIR.glob("*.tex")):
        text = tex_file.read_text()
        for m in LSTINPUTLISTING_RE.finditer(text):
            rel_path = m.group(1)
            # pdflatex resolves \lstinputlisting paths relative to main.tex's
            # own directory (DOCS_DIR), since that's where `make` invokes it
            # from -- not relative to the including chapter file.
            src_path = (DOCS_DIR / rel_path).resolve()
            location = f"{tex_file.name}:{line_of(text, m.start())} -> {rel_path}"
            if not src_path.exists():
                failures.append(Failure(location, "referenced file does not exist"))
                continue
            example_dir = src_path.parent
            example_dirs.setdefault(example_dir, location)

    checked = 0
    for example_dir, location in sorted(example_dirs.items()):
        makefile = example_dir / "Makefile"
        if not makefile.exists():
            failures.append(Failure(location, f"{example_dir.name} has no Makefile to check against"))
            continue
        checked += 1
        result = run(["make", "check"], cwd=example_dir)
        if result.returncode != 0:
            detail = (result.stdout + result.stderr)[-3000:]
            failures.append(Failure(location, f"`make check` failed in examples/{example_dir.name}", detail))

    print(f"Checked {checked} example director{'y' if checked == 1 else 'ies'} via `make check`.")
    for f in failures:
        print(f)
    if not failures and checked:
        print("OK: every worked-example program builds, runs, and matches its expected output.")
    return failures


def check_inline_listings():
    section("Verifying inline code listings (assembler)")
    failures = []
    checked = 0
    skipped = 0
    for tex_file in sorted(CHAPTERS_DIR.glob("*.tex")):
        text = tex_file.read_text()
        example_spans = find_example_spans(text)
        for m in LSTLISTING_RE.finditer(text):
            in_example = any(start <= m.start() < end for start, end in example_spans)
            opted_in = bool(SKIP_MARKER_RE.match(preceding_nonblank_line(text, m.start())))
            if not (in_example or opted_in):
                skipped += 1
                continue
            checked += 1
            body = m.group(1)
            location = f"{tex_file.name}:{line_of(text, m.start())}"
            with tempfile.TemporaryDirectory() as td:
                src_path = Path(td) / "listing.sasm"
                src_path.write_text(body)
                ok, detail = assemble(src_path, Path(td), do_link=False)
            if not ok:
                failures.append(Failure(location, "failed to assemble", detail))
    print(f"Checked {checked} inline listing(s), skipped {skipped} (Syntax/Operation templates).")
    for f in failures:
        print(f)
    if not failures and checked:
        print("OK: every verified inline listing assembles cleanly.")
    return failures


def render_pages(pages):
    section("Rendering pages for visual inspection")
    magick = subprocess.run(["which", "magick"], capture_output=True, text=True)
    if magick.returncode != 0:
        print("SKIPPED: ImageMagick ('magick') not found on PATH.")
        return
    pdf_path = DOCS_DIR / "SIRC-1-reference-manual.pdf"
    out_dir = Path(tempfile.mkdtemp(prefix="sirc-manual-qa-"))
    for page in pages:
        out_path = out_dir / f"page-{page}.png"
        result = run(["magick", "-density", "150", f"{pdf_path}[{page}]", str(out_path)])
        if result.returncode != 0:
            print(f"  page {page}: render failed -- {result.stderr.strip()}")
        else:
            print(f"  page {page} -> {out_path}")
    print(f"Rendered pages are in {out_dir}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true", help="reuse the existing PDF/log instead of rebuilding")
    parser.add_argument(
        "--render",
        metavar="N,N,...",
        help="0-indexed PDF page numbers to render to PNG for visual review (requires ImageMagick)",
    )
    args = parser.parse_args()

    all_failures = []
    all_failures += check_format()

    if not ensure_toolchain_built():
        print("\nCannot verify listings without a working assembler/linker build. Aborting.")
        sys.exit(1)

    if args.skip_build:
        section("Building PDF (make all)")
        print("SKIPPED (--skip-build)")
    else:
        built, build_failures = build_pdf()
        all_failures += build_failures
        if not built:
            print("\nBuild failed -- skipping listing verification, it would only add noise.")
            print_summary(all_failures)
            sys.exit(1)

    all_failures += check_references()
    all_failures += check_external_listings()
    all_failures += check_inline_listings()

    if args.render:
        pages = [int(p) for p in args.render.split(",")]
        render_pages(pages)

    print_summary(all_failures)
    sys.exit(1 if all_failures else 0)


def print_summary(failures):
    section("Summary")
    if failures:
        print(f"{len(failures)} problem(s) found:")
        for f in failures:
            print(f)
    else:
        print("All checks passed.")


if __name__ == "__main__":
    main()
