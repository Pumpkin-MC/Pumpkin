# PR title

feat(entity): implement sniffer brain and digging behavior

## Dеscription ·

> This description was drafted by an AI agent (Codex, GPT-6).

Moves sniffers from generic goals to the 26.3 Brain activities. Adds the sniff/search/dig/rise/happy cycle, reachable dig-site selection, gift-table seed drops, saved explored sites and sniff cooldown, egg breeding, digging dimensions, and state/event synchronization. Uses the existing Brain, navigation, loot and plugin events. Shared breeding finalization keeps normal offspring and persists the parents' cooldown through `Age`. Hatchlings use the sniffer's own baby age. Animal love mode clears at the accepted-damage stage, before death events, and sniffers retain the inherited age cleanup.

Implements the sniffer mob behavior tracked in #4216. Searching sounds, digging particles/audio, animations and quad-leash offsets are computed by the Java client from metadata and event 63.

Known boundaries: Pumpkin's synchronous navigation and block reads operate on loaded chunks. Sniffers use global tick reach for retained Brain/partner/breeder references; no narrow island audit or performance claim is included. Egg incubation delays remain a separate scheduler dependency in #4010.

## Tеsting ·

- Compared against official 26.3 server and client jars decompiled with Vineflower, tracing superclass ticks, Brain lifecycle, breeding, saved memory, navigation and metadata consumers.
- All affected packages: 718 tests passed with all features (696 core tests), including dig completion/interruption, entity-clock loot placement, reachable/explored-site filtering, saved positions/cooldown, egg breeding and normal cow offspring.
- `cargo fmt --check`, `cargo machete` and `typos` passed. Affected-package Clippy with all targets/features and `--no-deps` passed with the existing unknown `clippy::assert_is_empty` warning. Strict dependency-wide Clippy hits the existing `must_use_candidate` lint in `pumpkin-world/src/cylindrical_chunk_iterator.rs:67`.
- A scratch debug server and local 26.3 bot observed the natural sniff/search/dig/rise/happy cycle, event 63, the seed drop about six seconds into digging, saved Brain memories, egg breeding with exactly two parents, a hatchling at `Age: -47998`, accepted damage clearing love mode, and live temptation/damage interruption. The final server run exited normally.
- Real-client presentation, plugin cancellation with a live plugin, unloaded chunk boundaries and comparative performance have not been verified.

Play test with a real 26.3 Java client before opening:

1. Summon an adult on a large grass or dirt area and hold an empty hand. Expect sniffing, searching, digging, one seed/pod drop near its head, rising and a happy animation. Expect searching and digging audio, digging particles, a lower digging hitbox, and an eight-minute sniff cooldown after the completed dig.
2. Hold torchflower seeds near adults and babies. Expect temptation, with the baby following closer. Pitcher pods should not tempt or feed them. Feed babies and confirm growth; use a spawn egg on a sniffer and confirm the smaller baby size.
3. Feed two idle adults. Expect hearts and an egg item, with no baby sniffer entity. Save and reload; the parents should still have their breeding cooldown. Place the egg and confirm its hatchling is a baby with about 48000 ticks remaining in `Age`.
4. Damage a sniffer during sniffing/searching/digging. Expect idle/panic and no new sniff cooldown for an interrupted dig. Confirm leashed, riding, swimming, tempted and in-love sniffers do not start sniffing. Kill one while digging and confirm its state resets before death.
5. Save and reload after a completed dig. Check `Brain.memories` for explored positions and sniff cooldown. Confirm it avoids remembered ground and that a new entity does not resume a partial digging animation after reload.
6. Attach a sniffer to a compatible four-point leash holder and check the four client-rendered leads. Capture the full digging sequence with audio, the egg birth, and the interruption. Include the baby and digging hitboxes with F3+B if possible.

Capture: <attach screenshot or recording here>
