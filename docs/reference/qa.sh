#!/usr/bin/env bash
# SIRC-1 reference manual QA: runs the same `make` targets a CI pipeline
# would, in one command, for local use. Every check is a plain `make`
# target (docs/reference/Makefile, examples/*/Makefile,
# examples/manual-examples/Makefile) so this script is just a runner --
# there is no logic here that isn't equally callable directly with `make`.
set -euo pipefail
cd "$(dirname "$0")"

echo "== Format check =="
make format-check

echo
echo "== Build manual and check cross-references (make all) =="
make all

echo
echo "== Assembling listings extracted from the manual =="
make -C ../../examples/manual-examples objects

echo
echo "== Verifying worked-example programs referenced by Appendix D =="
# Discover example directories from the manual itself rather than hardcoding
# a list, so a newly-added \lstinputlisting is covered automatically.
grep -ohE '\\lstinputlisting(\[[^]]*\])?\{[^}]+\}' chapters/appendix-d-examples.tex \
	| sed -E 's/.*\{(.*)\}/\1/' \
	| xargs -n1 dirname \
	| sort -u \
	| while IFS= read -r dir; do
		echo "-- $dir --"
		make -C "$dir" clean check
	done

echo
echo "✓ All manual QA checks passed"
