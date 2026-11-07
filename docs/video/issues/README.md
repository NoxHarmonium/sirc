# PPU implementation issues (drafts)

Drafts of the GitHub issues for implementing the video device described by the manual in `docs/video`. Nothing here has been
posted. Review and edit the markdown first; each file is one issue, and `00-tracking-issue.md` is the overarching tracking issue.

| File | Issue |
| --- | --- |
| `00-tracking-issue.md` | Tracking issue with the plan, known bugs on `main` and conventions |
| `01-` ... `18-` | One issue each, grouped into milestones A-E (see the tracking issue) |
| `issues.tsv` | Index used by `post-issues.sh` |
| `post-issues.sh` | Posts everything with `gh`; **dry run unless you pass `--yes`** |

## Dependency order

```
PPU-01 registers/reset ─┬─ PPU-02 timing/PAL ─┬─ PPU-03 memory access/forced blank ─┐
                        │                      └─ PPU-04 interrupts                  │
                        │                                                           ▼
                        └────────────────────────────────── PPU-05 orthogonal engine (Mode 1)
                                                               ├─ PPU-06 16x16, map sizes, flips
                                                               ├─ PPU-07 2-bit orthogonal tiles
                                                               └─ PPU-08 compositor ─ PPU-09 output stage
PPU-10 sprites (needs 01, 03, 08) ─ PPU-11 shadow + overflow
PPU-12 iso geometry (pure model, no dependencies)
PPU-13 iso engine / Mode 2 (needs 05, 08, 12) ─┬─ PPU-14 iso options (also needs 07)
                                               ├─ PPU-15 status layer queue
                                               └─ PPU-16 Mode 3 (also needs 07)
PPU-17 tooling (needs 06, 07, 12)      PPU-18 examples and CI (needs 04, 05; ongoing)
```

PPU-12 is a good first issue: it is pure maths with exhaustive tests and nothing to wire up. PPU-01 and PPU-02 are the other
natural starts.

## Posting

```bash
./post-issues.sh                 # dry run: prints the gh commands
./post-issues.sh --yes           # creates the issues, then the tracking issue linking them
REPO=owner/name ./post-issues.sh --yes
```

The script needs `gh` authenticated for the repository. It creates a `ppu` label if missing, creates the 18 issues, then the
tracking issue with a task list of the real issue numbers.
