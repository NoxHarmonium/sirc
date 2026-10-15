# Pulls SIRC-1 assembly listings out of the reference manual's LaTeX source
# so they can be assembled with the real toolchain, catching syntax the
# manual claims is valid but isn't (see docs/reference/README.md's QA
# section for why and what counts as "verified").
#
# A listing is extracted if it is either:
#   - an inline \begin{lstlisting} block inside a \begin{example} float (the
#     Usage Examples scattered through the instruction chapters), or
#   - a bare \begin{lstlisting} block opted in with a "% qa:assemble"
#     comment on the line immediately before it (blank lines in between are
#     allowed; any other line cancels the opt-in).
#
# Everything else -- Syntax:/Operation: blocks using placeholder operands
# like "#opcode" or pseudocode like "if condition_true then" -- is not real
# SIRC-1 assembly and is left alone.
#
# One file is written per extracted listing, named
# <chapter-file>__<nearest-heading-slug>__<n>.sasm, into OUTDIR (default
# "extracted"). Deliberately POSIX-awk only (index/substr/gsub, no gawk-only
# match() array form) so this runs the same under BSD awk, gawk, and mawk.
#
# Usage: awk -v outdir=extracted -f extract-listings.awk chapters/*.tex

function slugify(s,    out) {
	out = tolower(s)
	gsub(/[^a-z0-9]+/, "-", out)
	gsub(/^-+/, "", out)
	gsub(/-+$/, "", out)
	if (out == "") out = "listing"
	return out
}

function extract_braced(line,    start, rest, e) {
	start = index(line, "{")
	if (start == 0) return ""
	rest = substr(line, start + 1)
	e = index(rest, "}")
	if (e == 0) return ""
	return substr(rest, 1, e - 1)
}

BEGIN {
	if (outdir == "") outdir = "extracted"
}

FNR == 1 {
	n = split(FILENAME, parts, "/")
	stem = parts[n]
	sub(/\.tex$/, "", stem)
	in_example = 0
	in_lstlisting = 0
	pending_marker = 0
	example_count = 0
	heading = stem
	outfile = ""
}

/\\begin\{example\}/ { in_example = 1 }
/\\end\{example\}/ { in_example = 0 }

/\\(sub)?section\{/ {
	h = extract_braced($0)
	if (h != "") heading = h
	pending_marker = 0
	next
}

# Per-instruction/per-mode entries are a more specific location than the
# \section they sit under (e.g. all of Chapter 17's meta-instructions share
# one "Overview" section), so prefer these when present. extract_braced
# grabs the *first* brace group on the line, which for
# "\begin{instructionbox}{COPI/COPR}{...}" is the environment name itself --
# strip that off first so it grabs the mnemonic argument instead.
/\\begin\{instructionbox\}\{/ {
	rest = $0
	sub(/^.*\\begin\{instructionbox\}/, "", rest)
	h = extract_braced(rest)
	if (h != "") heading = h
	pending_marker = 0
	next
}

/\\begin\{addrmodebox\}\{/ {
	rest = $0
	sub(/^.*\\begin\{addrmodebox\}/, "", rest)
	h = extract_braced(rest)
	if (h != "") heading = h
	pending_marker = 0
	next
}

/^[ \t]*%[ \t]*qa:assemble[ \t]*$/ {
	pending_marker = 1
	next
}

/\\begin\{lstlisting\}/ {
	if (in_example || pending_marker) {
		example_count++
		outfile = outdir "/" stem "__" slugify(heading) "__" example_count ".sasm"
		in_lstlisting = 1
		print outfile
	}
	pending_marker = 0
	next
}

/\\end\{lstlisting\}/ {
	if (in_lstlisting && outfile != "") close(outfile)
	in_lstlisting = 0
	outfile = ""
	next
}

in_lstlisting {
	print > outfile
	next
}

# Any other non-blank line cancels a pending opt-in marker: it wasn't
# immediately (modulo blank lines) before a \begin{lstlisting}.
!/^[ \t]*$/ {
	pending_marker = 0
}
