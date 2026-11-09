# PR title

feat(entity): implement sniffer brain and digging behavior

## Dеscription ·

> This description was drafted by an AI agent (Codex, GPT-6).

Moves sniffers to the 26.3 Brain activities for sniffing, searching, digging, rising and happy behavior. Adds reachable dig sites, gift-table seed drops, saved explored sites and cooldown, and state metadata and dimensions. Sniffers use the existing no-child breeding hook to lay an egg. Their parents keep a cooldown across reload, and accepted damage clears their love mode. Ordinary breeders keep their existing spawn and cooldown behavior. Hatching notifies clients and requires successful egg removal.

Implements the sniffer mob behavior tracked in #4216. The Java client handles searching sounds, digging particles and audio, animations and quad-leash offsets. The server sends state metadata and event 63, drops seeds and emits digging vibrations.

Pumpkin searches loaded chunks. A ticking sniffer uses global reach for its retained Brain and breeder references, so the world runs its non-player entity ticks serially. Vanilla-length egg incubation depends on the scheduler work in #4010.

## Tеsting ·

- Compared with official Java 26.3 server and client code for the Brain lifecycle, navigation, breeding and hatching, and the state and sound consumers.
- All 729 affected-package tests passed. Formatting, dependency, typo and affected-package Clippy checks passed. Clippy reports the existing unknown `clippy::assert_is_empty` lint.
- The egg breeding GameTest adapts Sean's scenario in `6094fd752`. It passed three consecutive runs on the default debug server, checking for exactly one egg and two parents.
- A 26.3 bot received two eating sounds from one successful feed. The server showed accepted damage clearing love, egg removal spawning a baby with `Age: -47935`, and frog breeding still setting pregnancy. The server stopped cleanly.
- Earlier 26.3 bot playtests covered natural digging, event 63, seed loot, save/reload, breeding, hatching and interruption. Real-client presentation, live plugin cancellation, unloaded chunk boundaries and the cost of serial entity ticking still need verification.

Play test with a real 26.3 Java client before opening:

1. Summon an adult on a large grass or dirt area and hold an empty hand. Expect sniffing, searching, digging, one seed/pod drop near its head, rising and a happy animation. Expect searching and digging audio, digging particles, a lower digging hitbox, and an eight-minute sniff cooldown after the completed dig.
2. Hold torchflower seeds near adults and babies. Expect temptation, with the baby following closer. Pitcher pods should not tempt or feed them. Feed babies and confirm growth; use a spawn egg on a sniffer and confirm the smaller baby size.
3. Feed two idle adults. Expect hearts and an egg item, with no baby sniffer entity. Save and reload; the parents should still have their breeding cooldown. Place the egg and confirm its hatchling is a baby with about 48000 ticks remaining in `Age`.
4. Damage a sniffer during sniffing/searching/digging. Expect idle/panic and no new sniff cooldown for an interrupted dig. Confirm leashed, riding, swimming, tempted and in-love sniffers do not start sniffing. Kill one while digging and confirm its state resets before death.
5. Save and reload after a completed dig. Check `Brain.memories` for explored positions and sniff cooldown. Confirm it avoids remembered ground and that a new entity does not resume a partial digging animation after reload.
6. Attach a sniffer to a compatible four-point leash holder and check the four client-rendered leads. Capture the full digging sequence with audio, the egg birth, and the interruption. Include the baby and digging hitboxes with F3+B if possible.

Capture: <attach screenshot or recording here>

Please follow our [Coding Guidelines](https://github.com/Pumpkin-MC/Pumpkin/blob/master/CONTRIBUTING.md#coding-guidelines) and read [When a PR May Be Closed](https://github.com/Pumpkin-MC/Pumpkin/blob/master/CONTRIBUTING.md#when-a-pr-may-be-closed)
