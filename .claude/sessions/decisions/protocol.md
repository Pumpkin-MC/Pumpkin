# Protocol — Decisions

## PROTO-001: Protocol DTO and Storage DTO Are Complementary
**Date:** 2026-02-07 | **Session:** protocol-003 | **Status:** DECIDED

Protocol DTO (`pumpkin-protocol/src/dto/`) handles wire-format version translation (per-client, per-connection). Storage DTO (`pumpkin-store`) handles persistence-format data access (per-world, shared). They operate at different layers and do not conflict.

For Tier 2+, protocol DTO may consume `GameDataStore` trait for item component-to-NBT conversion during packet translation.

**Responds to:** ARCH-020 compatibility question.
