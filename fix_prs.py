#!/usr/bin/env python3
"""Fix garbled PR bodies and CI formatting issues for all atomic PRs."""
import subprocess, os, json, tempfile

os.chdir(os.path.dirname(os.path.abspath(__file__)))

def run(cmd, check=True):
    r = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if check and r.returncode != 0:
        print(f"  WARN: {cmd}\n  {r.stderr.strip()[:200]}")
    return r.stdout.strip()

# ============================================================
# STEP 1: Fix PR bodies (garbled \n -> real newlines)
# ============================================================
pr_bodies = {
    1901: """## Summary
- Implements `/clone` command for copying blocks between regions
- Supports filtered, masked, and replace modes
- Uses vanilla translation keys

## Test plan
- [ ] `/clone 0 0 0 10 10 10 100 0 0` - clone a region
- [ ] Verify filtered/masked/replace modes work correctly""",

    1902: """## Summary
- Implements `/fillbiome` command for filling regions with specific biomes
- Uses vanilla translation keys

## Test plan
- [ ] `/fillbiome 0 0 0 10 10 10 minecraft:desert` - fill region with desert biome""",

    1903: """## Summary
- Implements `/spreadplayers` command for teleporting entities to random locations
- Uses vanilla translation keys

## Test plan
- [ ] `/spreadplayers 0 0 5 10 false @a` - spread all players
- [ ] Verify distance constraints are respected""",

    1904: """## Summary
- Implements `/random` command with value and roll subcommands
- OP level 0 (all players can use)
- Uses vanilla translation keys

## Test plan
- [ ] `/random value 1..6` - roll a random value
- [ ] Verify range bounds are enforced""",

    1905: """## Summary
- Implements `/scoreboard` command with objectives (add/remove/list) and players (set/add/remove/get/reset/list) subcommands
- Enhanced `Scoreboard` infrastructure with additional functionality

## Test plan
- [ ] `/scoreboard objectives add test dummy` - verify objective created
- [ ] `/scoreboard players set @s test 10` - verify score set
- [ ] `/scoreboard objectives list` - verify listing""",

    1906: """## Summary
- Implements `/team` command (add/remove/join/leave/modify/list)
- `Teams` system with `CUpdateTeams` packet for client synchronization
- Teams are sent to players on join

## Test plan
- [ ] `/team add red` - verify team created
- [ ] `/team join red @s` - verify player joins
- [ ] `/team modify red color red` - verify modification
- [ ] Verify teams sync to new joining players""",

    1907: """## Summary
- Implements `/teammsg` command for sending messages to team members
- OP level 0 (all players can use)

> **Note:** Depends on /team command being merged first.

## Test plan
- [ ] `/teammsg hello` - verify message sent to team members
- [ ] Verify non-team players cannot use the command""",

    1908: """## Summary
- Implements `/trigger` command for activating scoreboard triggers
- OP level 0 (all players can use)

> **Note:** Depends on /scoreboard command being merged first.

## Test plan
- [ ] `/trigger test` - verify trigger activation
- [ ] `/trigger test set 5` - verify set mode
- [ ] `/trigger test add 3` - verify add mode""",

    1909: """## Summary
- Implements `/attribute` command for querying, adding, removing, and setting entity attributes
- Uses vanilla translation keys

## Test plan
- [ ] `/attribute @s minecraft:generic.max_health get` - query attribute
- [ ] `/attribute @s minecraft:generic.max_health base set 40` - set attribute""",

    1910: """## Summary
- Implements `/execute` command with subcommands (as, at, positioned, if, unless, run, etc.)
- Uses vanilla translation keys

## Test plan
- [ ] `/execute as @a run say hello` - execute as all players
- [ ] `/execute at @s positioned 0 100 0 run tp @s ~ ~ ~` - positioned execution
- [ ] `/execute if entity @e[type=zombie] run say zombies exist` - conditional""",

    1911: """## Summary
- Implements `/item` command for manipulating items in entity and block inventories
- Uses vanilla translation keys

## Test plan
- [ ] `/item replace entity @s weapon.mainhand with diamond_sword` - set item
- [ ] `/item modify entity @s weapon.mainhand enchant_randomly` - modify item""",

    1912: """## Summary
- Replace hardcoded translation key strings with `pumpkin_data::translation` constants
- Affected commands: fill, setworldspawn, spawnpoint, teleport

## Test plan
- [ ] `cargo clippy -p pumpkin --all-targets --all-features -- -Dwarnings`
- [ ] Verify affected commands still work correctly""",
}

print("=== STEP 1: Fixing PR bodies ===")
for pr_num, body in pr_bodies.items():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False, encoding='utf-8') as f:
        f.write(body)
        tmp = f.name
    run(f'gh pr edit {pr_num} --repo Pumpkin-MC/Pumpkin --body-file "{tmp}"')
    os.unlink(tmp)
    print(f"  PR #{pr_num} body updated")

# ============================================================
# STEP 2: Fix formatting on all branches (cargo fmt + push)
# ============================================================
print("\n=== STEP 2: Fixing formatting ===")

branches = [
    'feat/cmd-clone-atomic',
    'feat/cmd-fillbiome-atomic',
    'feat/cmd-spreadplayers-atomic',
    'feat/cmd-random-atomic',
    'feat/cmd-scoreboard-atomic',
    'feat/cmd-team-atomic',
    'feat/cmd-teammsg-atomic',
    'feat/cmd-trigger-atomic',
    'feat/cmd-attribute-atomic',
    'feat/cmd-execute-atomic',
    'feat/cmd-item-atomic',
    'fix/translation-keys',
]

for branch in branches:
    print(f"\n  Fixing: {branch}")
    run(f'git checkout {branch}', check=False)

    # Run cargo fmt
    fmt_result = subprocess.run('cargo fmt --all', shell=True, capture_output=True, text=True)
    if fmt_result.returncode != 0:
        print(f"    cargo fmt failed: {fmt_result.stderr[:200]}")
        continue

    # Check if there are changes
    status = run('git status --porcelain')
    if status:
        run('git add -A')
        run('git commit -m "style: apply cargo fmt"')
        run(f'git push fork {branch} --force')
        print(f"    Formatted and pushed")
    else:
        print(f"    Already formatted")

# ============================================================
# STEP 3: Fix fillbiome.rs compile error (Arc borrow)
# ============================================================
print("\n=== STEP 3: Fixing fillbiome compile error ===")
run('git checkout feat/cmd-fillbiome-atomic', check=False)

# Read the file and fix the Arc borrow issue
fpath = 'pumpkin/src/command/commands/fillbiome.rs'
if os.path.exists(fpath):
    with open(fpath, 'r') as f:
        content = f.read()

    # The error is at line 77: trying to mutably borrow through Arc
    # Need to check the exact code and fix it
    print(f"  fillbiome.rs line ~77 area:")
    lines = content.split('\n')
    for i in range(max(0,72), min(len(lines), 88)):
        print(f"    {i+1}: {lines[i]}")

print("\nDone with automated fixes. Check fillbiome manually if needed.")
run('git checkout origin/master --detach', check=False)
