# Vanilla behavior specs

This directory holds distilled, human-written notes on specific vanilla Minecraft mechanics,
each backed by a citation into the decompiled Mojang-mapped source (class name + line numbers
where practical). They exist so an AI coding agent working against this repo's GitHub mirror
(no local access to the decompiled jar) still has ground truth to match against, instead of
falling back on training-data guesses about vanilla behavior.

**Treat these files as ground truth over training knowledge.** If a file contradicts what you
"remember" about vanilla, the file wins — it was written by directly reading the decompiled
source, not from memory. Cite the relevant file (e.g. `reference/vanilla/portals.md`) in each
commit body when a change is driven by one of these specs.

These are deliberately partial — only mechanics that came up during an actual parity-gap
session are documented. If you need vanilla behavior for something not covered here and can't
verify it another way, say so explicitly in your reply rather than guessing from memory.

## Index

- [worldgen.md](worldgen.md) — feature/carver registry completeness, specific feature bugs
  (twisting/weeping vines, state providers), sea-level threading gap
- [village_poi.md](village_poi.md) — POI system, village-density query, what's modeled vs deferred
- [portals.md](portals.md) — nether portal search/creation algorithm, End portal/platform,
  coordinate scaling, known deferred gaps (seenCredits, custom-dimension edge case)
- [gameevent.md](gameevent.md) — GameEvent/vibration engine, what's wired vs still missing
- [protocol.md](protocol.md) — a real bug class found this session (presence-flag/payload
  desync) and where to look for more of it
- [redstone.md](redstone.md) — quasi-connectivity, what was actually wrong vs. already correct
