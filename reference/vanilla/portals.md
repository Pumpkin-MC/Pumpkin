# Dimensions and portals

Source: `net.minecraft.world.level.portal.{PortalForcer,PortalShape}`,
`net.minecraft.world.level.block.{NetherPortalBlock,EndPortalBlock,EndPortalFrameBlock}`,
`net.minecraft.world.level.levelgen.feature.EndPlatformFeature`, decompiled 26.2 mappings.

## Fixed this session (`pumpkin/src/world/portal/nether.rs`) — for context, don't re-fix

1. **Existing-portal search radius.** `PortalForcer.java:23-24,43`:
   `NETHER_PORTAL_RADIUS = 16`, `OVERWORLD_PORTAL_RADIUS = 128`, selected by destination
   dimension (`toNether ? NETHER_PORTAL_RADIUS : OVERWORLD_PORTAL_RADIUS`). Nether-bound
   teleports search only 16 blocks; only Overworld-bound teleports use 128. Pumpkin had both set
   to 128 — Nether-bound teleports were searching 8x too wide, able to snap onto a far-away
   existing portal instead of building a closer new one. Fixed:
   `SEARCH_RADIUS_NETHER = 16`, selector compares `dest_world.dimension` against
   `Dimension::THE_NETHER` directly (previously used `has_ceiling`, which is also true for
   `OVERWORLD_CAVES` — harmless today since nothing teleports there via portal, but imprecise).
2. **New-portal search area.** `PortalForcer.java:63`: `BlockPos.spiralAround(origin, 16, ...)`
   — always radius 16 regardless of destination dimension. Pumpkin used a hardcoded `-32..=32`
   (radius 32). Fixed to a named `CREATE_PORTAL_SEARCH_RADIUS = 16`.
3. **Missing `deltaY <= 0` acceptance branch (the highest-impact of the three).**
   `PortalForcer.java:78-79`: `int deltaY = firstEmptyY - y; if (deltaY <= 0 || deltaY >= 3) { ... }`.
   Pumpkin only implemented the `>= 3` half. The common "there's exactly one usable air block
   above solid ground" case (`deltaY <= 0`) was silently rejected, degrading portal placement to
   the floating-obsidian-box fallback far more often than vanilla. Fixed to match exactly.

## Verified correct, no changes needed (checked this session)

- Coordinate scaling: `coordinate_scale = 8` for Nether, `= 1` for Overworld/End, matches
  `DimensionType.getTeleportationScale` (`DimensionType.java:108-111`).
- Portal-frame validation (`get_lower_cornor`/`get_width`/`get_height`/`is_horizontal_frame_valid`)
  closely mirrors `PortalShape.calculateBottomLeft`/`calculateWidth`/`calculateHeight`/
  `getDistanceUntilTop`, including the `pos.y - 21` min-y clamp and `MIN/MAX_WIDTH`/`HEIGHT` bounds.
- Portal cooldown: player 10 ticks, other entities 300 ticks, matches
  `Entity.getDimensionChangingDelay`; reset-on-reentry and decay-by-4-per-tick both present and
  correctly gate re-teleport.
- End platform (5×5 obsidian, 3 layers of air, player Y offset `-1.0` vs. other entities' `+0.0`)
  matches `EndPlatformFeature.createEndPlatform`/`EndPortalBlock.getPortalDestination`
  (`EndPortalBlock.java:79-96`) exactly.
- End portal frame → end-portal creation (12-eye check, 3×3 `END_PORTAL` placement) matches the
  `EndPortalFrameBlock` pattern check.
- `search_max_y`/`top_y_limit` conditioning on `has_ceiling` rather than unconditionally using
  `logical_height` is behaviorally identical for the stock dimension table (only the Nether's
  `logical_height` differs from its `height`, and only the Nether has `has_ceiling = true` among
  real portal destinations) — this is a shortcut that happens to be safe today, not a bug, but
  it would break for a custom dimension with `has_ceiling = false` and `logical_height < height`.

## Real, confirmed divergences NOT yet fixed — pick one of these next

- **No `seenCredits` tracking (real, user-visible gap).** `EndPortalBlock.java:64-71`: entering
  the end-exit-portal while `!player.seenCredits` triggers `showEndCredits()` and does **not**
  teleport that time — `setAsInsidePortal` is skipped entirely on the first crossing. Only on
  subsequent crossings does normal teleport happen, and even then vanilla never sends `WinGame`
  again (that packet is only for the credits path, sent once). Pumpkin's
  `pumpkin/src/world/portal/mod.rs:137-158` unconditionally sends `WinGame` and teleports on
  every single crossing. Confirmed via `grep`: no `seen_credits` field exists anywhere in
  Pumpkin. This needs new persistent per-player state (survives death/relog, matches vanilla's
  player-data NBT field) plus a client-flow change (skip teleport once, but still trigger the
  credits screen) — not a one-line fix, but well-scoped as its own unit of work.
- **Fluid-state check for portal-air validity.** `PortalForcer.java:160`:
  `blockState.canBeReplaced() && blockState.getFluidState().isEmpty()`. Pumpkin's
  `is_valid_portal_air` uses `state.replaceable() && !state.is_liquid()`. These are NOT
  necessarily equivalent for a waterlogged replaceable block (non-empty fluid state in vanilla,
  but Pumpkin's `is_liquid()` may or may not report true depending on how waterlogging is
  represented — not confirmed as an active bug, needs checking Pumpkin's actual waterlogging
  model before deciding this is broken).
- **Missing empty-result handling for the create-portal fallback (latent panic, not currently
  reachable).** `PortalForcer.java:107-112`: if `maxStartY < minStartY` (only possible for a
  custom dimension with a very short `logical_height`), vanilla returns `Optional.empty()` and
  aborts the teleport with a logged error. Pumpkin's fallback in `find_safe_location` computes
  `target_pos.0.y.clamp(min_y.max(70), top_y_limit - 9)` unconditionally — `clamp` panics if
  `min > max`. Unreachable with the stock Nether/Overworld/End dimension table (all have room
  ≥ 70), but would panic for a custom dimension. A defensive early-return matching vanilla's
  graceful-failure behavior is the right fix, not just a saturating clamp — match the abort
  behavior, don't just suppress the panic.
