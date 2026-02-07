# Core Agent Reference Data

> Extracted 2026-02-07 from Pumpkin specs + codebase analysis.
> Sources: `1.21.4/summary/commands.json`, Bukkit API ref, Pumpkin source tree.

---

## Vanilla Commands (from commands.json)

All 83 top-level vanilla commands in 1.21.4:

```
advancement    attribute      ban            ban-ip         banlist
bossbar        clear          clone          damage         data
datapack       debug          defaultgamemode deop          difficulty
effect         enchant        execute        experience     fill
fillbiome      forceload      function       gamemode       gamerule
give           help           item           jfr            kick
kill           list           locate         loot           me
msg            op             pardon         pardon-ip      particle
perf           place          playsound      publish        random
recipe         reload         return         ride           rotate
save-all       save-off       save-on        say            schedule
scoreboard     seed           setblock       setidletimeout setworldspawn
spawnpoint     spectate       spreadplayers  stop           stopsound
summon         tag            team           teammsg        teleport
tell           tellraw        tick           time           title
tm             tp             transfer       trigger        w
weather        whitelist      worldborder    xp
```

Aliases: `tell`/`w`/`msg`, `tp`/`teleport`, `tm`/`teammsg`, `xp`/`experience`.

---

## Pumpkin Commands (What's Implemented)

**89 command .rs files** found in `pumpkin/src/command/` tree.
**51 command implementations** in `pumpkin/src/command/commands/`:

```
ban            banip          banlist        bossbar        clear
damage         data           defaultgamemode deop          difficulty
effect         enchant        experience     fill           gamemode
gamerule       give           help           kick           kill
list           me             mod.rs         msg            op
pardon         pardonip       particle       playsound      plugin
plugins        pumpkin        rotate         say            seed
setblock       setidletimeout setworldspawn  spawnpoint     stop
stopsound      summon         teleport       tellraw        tick
time           title          transfer       weather        whitelist
worldborder
```

### Implemented vs Vanilla -- Comparison

| Status | Commands |
|---|---|
| **Implemented** (41 vanilla) | advancement(no), attribute(no), ban, ban-ip, banlist, bossbar, clear, damage, data, defaultgamemode, deop, difficulty, effect, enchant, experience, fill, gamemode, gamerule, give, help, kick, kill, list, me, msg, op, pardon, pardon-ip, particle, playsound, rotate, say, seed, setblock, setidletimeout, setworldspawn, spawnpoint, stop, stopsound, summon, teleport, tellraw, tick, time, title, transfer, weather, whitelist, worldborder |
| **Pumpkin-only** (not in vanilla) | plugin, plugins, pumpkin |
| **Missing from vanilla** (~34) | advancement, attribute, clone, datapack, debug, execute, fillbiome, forceload, function, item, jfr, locate, loot, perf, place, publish, random, recipe, reload, return, ride, save-all, save-off, save-on, schedule, scoreboard, spectate, spreadplayers, tag, team, teammsg, trigger |

### Missing Command Priority Assessment

| Priority | Commands | Reason |
|---|---|---|
| **High** | execute, clone, scoreboard, tag, team | Core gameplay and map-making |
| **Medium** | locate, datapack, recipe, reload, function | Server administration |
| **Medium** | save-all, save-off, save-on | World persistence |
| **Low** | fillbiome, forceload, spreadplayers, spectate | Niche use cases |
| **Low** | jfr, perf, debug, publish | Diagnostics / LAN |
| **Low** | loot, item, place, ride, return | Advanced / newer commands |
| **Deferred** | schedule, random, trigger | Need scheduler / scoreboard first |

---

## Server Tick Loop

### File: `pumpkin/src/server/ticker.rs` (61 lines)

The tick loop is an async tokio task:

```
loop:
  1. Check SHOULD_STOP atomic flag
  2. Record tick_start_time
  3. manager.tick() -- updates frozen tick state
  4. If sprinting:
     a. start_sprint_tick_work()
     b. server.tick().await
     c. end_sprint_tick_work() -- if sprint done, finish_tick_sprint()
  5. Else:
     a. server.tick().await
  6. Record tick duration (nanos)
  7. Sleep for remaining time in tick interval
  8. Default tick rate: 20 TPS (50ms per tick)
```

### File: `pumpkin/src/server/tick_rate_manager.rs` (224 lines)

Manages tick rate, freeze, sprint, and step-forward:

**State fields:**
- `tickrate: AtomicCell<f32>` -- default 20.0
- `nanoseconds_per_tick: AtomicI64` -- derived from tickrate
- `frozen_ticks_to_run: AtomicI32` -- for `/tick step` command
- `run_game_elements: AtomicBool` -- false when frozen (no step pending)
- `is_frozen: AtomicBool` -- game pause state
- `remaining_sprint_ticks: AtomicI64` -- for `/tick sprint`
- Sprint timing fields for performance reporting

**Key methods:**
- `set_tick_rate(rate)` -- change TPS, notify clients via CTickingState
- `set_frozen(frozen)` -- pause/unpause game, notify clients
- `step_game_if_paused(ticks)` -- `/tick step N` when frozen
- `request_game_to_sprint(ticks)` -- `/tick sprint N`
- `finish_tick_sprint()` -- reports TPS/MSPT, restores previous frozen state

### Full Server Directory: `pumpkin/src/server/` (1,901 lines total)

| File | Lines | Purpose |
|---|---|---|
| mod.rs | 958 | Server struct, initialization, tick(), world management |
| tick_profiler.rs | 379 | Tick performance profiling |
| tick_rate_manager.rs | 224 | Tick rate / freeze / sprint control |
| connection_cache.rs | 170 | Cached server status / MOTD |
| key_store.rs | 80 | RSA key pair for encryption |
| ticker.rs | 61 | Main tick loop driver |
| seasonal_events.rs | 29 | April Fools / Halloween events |

---

## Scheduler API Reference (from Bukkit)

The Bukkit `BukkitScheduler` interface defines the standard plugin scheduler API. Key methods:

### Synchronous (Main Thread)

| Method | Description |
|---|---|
| `scheduleSyncDelayedTask(plugin, task, delay)` | Run `task` after `delay` server ticks on main thread. Returns task ID. |
| `scheduleSyncDelayedTask(plugin, task)` | Run `task` on next tick (delay=0). Returns task ID. |
| `scheduleSyncRepeatingTask(plugin, task, delay, period)` | Run `task` every `period` ticks after initial `delay`. Returns task ID. |
| `runTask(plugin, task)` | Run on next server tick. Returns BukkitTask. |

### Asynchronous (Worker Thread)

| Method | Description |
|---|---|
| `runTaskAsynchronously(plugin, task)` | Run on async thread immediately. |
| `runTaskLaterAsynchronously(plugin, task, delay)` | Run on async thread after `delay` ticks. |
| `runTaskTimerAsynchronously(plugin, task, delay, period)` | Repeat on async thread. |

**Warning**: Async tasks must NOT access Bukkit API. Thread-safety is the plugin's responsibility.

### Task Management

| Method | Description |
|---|---|
| `cancelTask(taskId)` | Remove task by ID. |
| `cancelTasks(plugin)` | Remove all tasks for a plugin. |
| `isCurrentlyRunning(taskId)` | Check if task thread is alive. |
| `isQueued(taskId)` | Check if task is pending execution. |
| `getActiveWorkers()` | List all running async tasks. |
| `getPendingTasks()` | List all queued tasks. |
| `callSyncMethod(plugin, callable)` | Run Callable on main thread, return Future. |

### Pumpkin Equivalent

Pumpkin does **not** currently have a plugin scheduler. The tick loop exists (`ticker.rs`) but there is no API for plugins to schedule delayed/repeating tasks. This is listed as pending work.

**Design considerations for Pumpkin scheduler:**
- Rust async (tokio) instead of Java threads
- Plugin owns tasks via plugin ID, not Java Plugin object
- `Arc<dyn Fn()>` or similar for task closures
- Tick-based delays map naturally to the existing tick loop
- Async tasks can use `tokio::spawn` but need Send + Sync bounds

---

## Gap Analysis

### Commands

- **49 of 83** vanilla commands implemented (~59%)
- 3 Pumpkin-specific commands added (plugin, plugins, pumpkin)
- Highest priority gaps: `execute`, `clone`, `scoreboard`, `tag`, `team`
- `execute` is critical -- it is the most complex command (subcommand chaining, conditionals, entity selectors)

### Server Infrastructure

- Tick loop is functional with rate control, freeze, sprint, and step
- Tick profiler exists (379 lines)
- **No plugin scheduler** -- plugins cannot schedule delayed or repeating tasks
- **No world save commands** -- `save-all`, `save-off`, `save-on` are missing
- **No datapack system** -- `datapack`, `reload`, `function` commands depend on this
- **No scoreboard system** -- `scoreboard`, `tag`, `team`, `trigger` commands depend on this
- **No loot table command integration** -- `loot` command needs loot table engine

### Tick Loop Health

- Clean async design using tokio
- Proper sprint/freeze/step support matching vanilla `/tick` command
- Tick duration tracking in place
- Missing: scheduled tick callbacks for blocks (repeaters, observers, etc.)
- Missing: entity tick scheduling (mob AI, projectile updates)

### Scheduler Priority

Building a plugin scheduler should be high priority because:
1. Many plugins need delayed/repeating tasks (economy, minigames, teleport warmup)
2. The tick loop already exists as an integration point
3. Bukkit's API is well-understood and can be adapted to Rust idioms
4. Without it, plugins are limited to event-driven logic only
