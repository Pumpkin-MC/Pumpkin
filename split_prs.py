#!/usr/bin/env python3
"""
Split bundled PRs into atomic PRs.
Creates individual branches for each command, pushes to fork, creates PRs.
"""

import subprocess
import sys
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

def run(cmd, check=True):
    """Run a shell command and return output."""
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"ERROR: {cmd}\n{result.stderr}")
        return None
    return result.stdout.strip()

def insert_mod_declaration(content, mod_name):
    """Insert 'mod <name>;' in alphabetical order in mod declarations."""
    lines = content.split('\n')
    insert_idx = None
    for i, line in enumerate(lines):
        if line.strip().startswith('mod ') and line.strip().endswith(';'):
            existing = line.strip().replace('mod ', '').replace(';', '').replace('pub ', '')
            if existing > mod_name and insert_idx is None:
                insert_idx = i
    if insert_idx is not None:
        lines.insert(insert_idx, f'mod {mod_name};')
    return '\n'.join(lines)

def insert_dispatcher_register(content, mod_name, op_level):
    """Insert dispatcher.register() call in the right OP level section."""
    lines = content.split('\n')

    if op_level == 0:
        # Insert before the "// Two" comment
        marker = '    // Two'
    elif op_level == 2:
        # Insert before the "// Three" comment
        marker = '    // Three'
    else:
        marker = '    // Four'

    insert_idx = None
    for i, line in enumerate(lines):
        if line.strip() == marker.strip():
            insert_idx = i
            break

    if insert_idx is not None:
        reg_lines = [
            f'    dispatcher.register(',
            f'        {mod_name}::init_command_tree(),',
            f'        "minecraft:command.{mod_name}",',
            f'    );',
        ]
        for j, rl in enumerate(reg_lines):
            lines.insert(insert_idx + j, rl)

    return '\n'.join(lines)

def insert_permission(content, mod_name, description, op_level):
    """Insert permission registration in the right function."""
    lines = content.split('\n')

    if op_level == 0:
        func_name = 'fn register_level_0_permissions'
    elif op_level == 2:
        func_name = 'fn register_level_2_permissions'
    else:
        func_name = 'fn register_level_3_permissions'

    # Find the LAST .unwrap(); before the closing } of the function
    func_start = None
    for i, line in enumerate(lines):
        if func_name in line:
            func_start = i
            break

    if func_start is None:
        return content

    # Find the closing } of this function
    brace_count = 0
    func_end = None
    for i in range(func_start, len(lines)):
        brace_count += lines[i].count('{') - lines[i].count('}')
        if brace_count == 0 and i > func_start:
            func_end = i
            break

    if func_end is None:
        return content

    perm_level = 'PermissionDefault::Allow' if op_level == 0 else f'PermissionDefault::Op(PermissionLvl::Two)'

    perm_lines = [
        '    registry',
        '        .register_permission(Permission::new(',
        f'            "minecraft:command.{mod_name}",',
        f'            "{description}",',
        f'            {perm_level},',
        '        ))',
        '        .unwrap();',
    ]

    for j, pl in enumerate(perm_lines):
        lines.insert(func_end + j, pl)

    return '\n'.join(lines)

def create_atomic_branch(branch_name, source_branch, command_files, infra_files,
                          mod_name, op_level, perm_desc, commit_msg, pr_title, pr_body):
    """Create an atomic branch with a single command."""
    print(f"\n{'='*60}")
    print(f"Creating: {branch_name}")
    print(f"{'='*60}")

    # Delete branch if exists locally
    run(f'git branch -D {branch_name}', check=False)

    # Create branch from master
    result = run(f'git checkout -b {branch_name} origin/master')
    if result is None:
        print(f"SKIP: Could not create branch {branch_name}")
        run('git checkout origin/master --detach', check=False)
        return False

    # Copy command file(s)
    for cf in command_files:
        run(f'git checkout {source_branch} -- {cf}')

    # Copy infrastructure files
    for inf in infra_files:
        run(f'git checkout {source_branch} -- {inf}')

    # Modify mod.rs if mod_name is provided
    if mod_name:
        mod_rs_path = 'pumpkin/src/command/commands/mod.rs'
        with open(mod_rs_path, 'r') as f:
            content = f.read()

        # Check if mod already declared (for existing commands like execute, attribute, item)
        if f'mod {mod_name};' not in content:
            content = insert_mod_declaration(content, mod_name)

        content = insert_dispatcher_register(content, mod_name, op_level)
        content = insert_permission(content, mod_name, perm_desc, op_level)

        with open(mod_rs_path, 'w') as f:
            f.write(content)

    # Stage all changes
    run('git add -A')

    # Commit (no Claude co-author)
    run(f'git commit -m "{commit_msg}"')

    # Push to fork
    run(f'git push fork {branch_name} --force')

    # Create PR
    pr_cmd = f'gh pr create --repo Pumpkin-MC/Pumpkin --head WhiteProject1:{branch_name} --base master --title "{pr_title}" --body "{pr_body}"'
    pr_result = run(pr_cmd, check=False)
    if pr_result:
        print(f"PR created: {pr_result}")
    else:
        print(f"PR creation skipped (may already exist)")

    return True


def main():
    # Ensure we're on a clean state
    run('git fetch origin master --quiet')

    # ================================================================
    # GROUP 1: From #1875 (world-manipulation) - 4 independent commands
    # ================================================================

    create_atomic_branch(
        branch_name='feat/cmd-clone-atomic',
        source_branch='feat/cmd-world-manipulation',
        command_files=['pumpkin/src/command/commands/clone.rs'],
        infra_files=[],
        mod_name='clone',
        op_level=2,
        perm_desc='Copies blocks from one region to another',
        commit_msg='feat: implement /clone command',
        pr_title='feat: implement /clone command',
        pr_body='## Summary\\n- Implements `/clone` command for copying blocks between regions\\n- Supports filtered, masked, and replace modes\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/clone 0 0 0 10 10 10 100 0 0` - clone a region\\n- [ ] Verify filtered/masked/replace modes work correctly',
    )

    create_atomic_branch(
        branch_name='feat/cmd-fillbiome-atomic',
        source_branch='feat/cmd-world-manipulation',
        command_files=['pumpkin/src/command/commands/fillbiome.rs'],
        infra_files=[],
        mod_name='fillbiome',
        op_level=2,
        perm_desc='Fills a region with a specific biome',
        commit_msg='feat: implement /fillbiome command',
        pr_title='feat: implement /fillbiome command',
        pr_body='## Summary\\n- Implements `/fillbiome` command for filling regions with specific biomes\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/fillbiome 0 0 0 10 10 10 minecraft:desert` - fill region with desert biome',
    )

    create_atomic_branch(
        branch_name='feat/cmd-spreadplayers-atomic',
        source_branch='feat/cmd-world-manipulation',
        command_files=['pumpkin/src/command/commands/spreadplayers.rs'],
        infra_files=[],
        mod_name='spreadplayers',
        op_level=2,
        perm_desc='Teleports entities to random surface locations in an area',
        commit_msg='feat: implement /spreadplayers command',
        pr_title='feat: implement /spreadplayers command',
        pr_body='## Summary\\n- Implements `/spreadplayers` command for teleporting entities to random locations\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/spreadplayers 0 0 5 10 false @a` - spread all players\\n- [ ] Verify distance constraints are respected',
    )

    create_atomic_branch(
        branch_name='feat/cmd-random-atomic',
        source_branch='feat/cmd-world-manipulation',
        command_files=['pumpkin/src/command/commands/random.rs'],
        infra_files=[],
        mod_name='random',
        op_level=0,
        perm_desc='Draw a random value or control random sequences',
        commit_msg='feat: implement /random command',
        pr_title='feat: implement /random command',
        pr_body='## Summary\\n- Implements `/random` command with value and roll subcommands\\n- OP level 0 (all players)\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/random value 1..6` - roll a random value\\n- [ ] Verify range bounds are enforced',
    )

    # ================================================================
    # GROUP 2: From #1874 (scoreboard-teams) - 4 commands with infra
    # ================================================================

    create_atomic_branch(
        branch_name='feat/cmd-scoreboard-atomic',
        source_branch='feat/cmd-scoreboard-teams',
        command_files=['pumpkin/src/command/commands/scoreboard.rs'],
        infra_files=['pumpkin/src/world/scoreboard.rs', 'pumpkin/src/world/mod.rs'],
        mod_name='scoreboard',
        op_level=2,
        perm_desc='Manages scoreboard objectives and players',
        commit_msg='feat: implement /scoreboard command\\n\\nAdd /scoreboard command with objectives and players management.\\nIncludes enhanced scoreboard infrastructure.',
        pr_title='feat: implement /scoreboard command',
        pr_body='## Summary\\n- Implements `/scoreboard` command with objectives (add/remove/list) and players (set/add/remove/get/reset/list) subcommands\\n- Enhanced `Scoreboard` infrastructure with additional functionality\\n\\n## Test plan\\n- [ ] `/scoreboard objectives add test dummy` - verify objective created\\n- [ ] `/scoreboard players set @s test 10` - verify score set\\n- [ ] `/scoreboard objectives list` - verify listing',
    )

    create_atomic_branch(
        branch_name='feat/cmd-team-atomic',
        source_branch='feat/cmd-scoreboard-teams',
        command_files=['pumpkin/src/command/commands/team.rs'],
        infra_files=[
            'pumpkin/src/world/teams.rs',
            'pumpkin/src/world/mod.rs',
            'pumpkin-protocol/src/java/client/play/update_teams.rs',
            'pumpkin-protocol/src/java/client/play/mod.rs',
        ],
        mod_name='team',
        op_level=2,
        perm_desc='Controls teams',
        commit_msg='feat: implement /team command\\n\\nAdd /team command with create, modify, remove, join, leave operations.\\nIncludes Teams system with CUpdateTeams packet for client sync.',
        pr_title='feat: implement /team command',
        pr_body='## Summary\\n- Implements `/team` command (add/remove/join/leave/modify/list)\\n- `Teams` system with `CUpdateTeams` packet for client synchronization\\n- Teams are sent to players on join\\n\\n## Test plan\\n- [ ] `/team add red` - verify team created\\n- [ ] `/team join red @s` - verify player joins\\n- [ ] `/team modify red color red` - verify modification\\n- [ ] Verify teams sync to new joining players',
    )

    create_atomic_branch(
        branch_name='feat/cmd-teammsg-atomic',
        source_branch='feat/cmd-scoreboard-teams',
        command_files=['pumpkin/src/command/commands/teammsg.rs'],
        infra_files=[],
        mod_name='teammsg',
        op_level=0,
        perm_desc="Sends a message to all players on the sender's team",
        commit_msg='feat: implement /teammsg command',
        pr_title='feat: implement /teammsg command',
        pr_body='## Summary\\n- Implements `/teammsg` command for sending messages to team members\\n- OP level 0 (all players)\\n\\n> **Note:** Depends on /team command being merged first.\\n\\n## Test plan\\n- [ ] `/teammsg hello` - verify message sent to team members\\n- [ ] Verify non-team players cannot use the command',
    )

    create_atomic_branch(
        branch_name='feat/cmd-trigger-atomic',
        source_branch='feat/cmd-scoreboard-teams',
        command_files=['pumpkin/src/command/commands/trigger.rs'],
        infra_files=[],
        mod_name='trigger',
        op_level=0,
        perm_desc='Sets a trigger to be activated',
        commit_msg='feat: implement /trigger command',
        pr_title='feat: implement /trigger command',
        pr_body='## Summary\\n- Implements `/trigger` command for activating scoreboard triggers\\n- OP level 0 (all players)\\n\\n> **Note:** Depends on /scoreboard command being merged first.\\n\\n## Test plan\\n- [ ] `/trigger test` - verify trigger activation\\n- [ ] `/trigger test set 5` - verify set mode\\n- [ ] `/trigger test add 3` - verify add mode',
    )

    # ================================================================
    # GROUP 3: From #1876 (execute-attribute-item) - 3 commands + fixes
    # ================================================================

    create_atomic_branch(
        branch_name='feat/cmd-attribute-atomic',
        source_branch='feat/cmd-execute-attribute-item',
        command_files=['pumpkin/src/command/commands/attribute.rs'],
        infra_files=[],
        mod_name='attribute',
        op_level=2,
        perm_desc='Queries, adds, removes, or sets an entity attribute',
        commit_msg='feat: implement /attribute command',
        pr_title='feat: implement /attribute command',
        pr_body='## Summary\\n- Implements `/attribute` command for querying, adding, removing, and setting entity attributes\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/attribute @s minecraft:generic.max_health get` - query attribute\\n- [ ] `/attribute @s minecraft:generic.max_health base set 40` - set attribute',
    )

    create_atomic_branch(
        branch_name='feat/cmd-execute-atomic',
        source_branch='feat/cmd-execute-attribute-item',
        command_files=['pumpkin/src/command/commands/execute.rs'],
        infra_files=[],
        mod_name='execute',
        op_level=2,
        perm_desc='Executes a command',
        commit_msg='feat: implement /execute command',
        pr_title='feat: implement /execute command',
        pr_body='## Summary\\n- Implements `/execute` command with subcommands (as, at, positioned, if, unless, run, etc.)\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/execute as @a run say hello` - execute as all players\\n- [ ] `/execute at @s positioned 0 100 0 run tp @s ~ ~ ~` - positioned execution\\n- [ ] `/execute if entity @e[type=zombie] run say zombies exist` - conditional',
    )

    create_atomic_branch(
        branch_name='feat/cmd-item-atomic',
        source_branch='feat/cmd-execute-attribute-item',
        command_files=['pumpkin/src/command/commands/item.rs'],
        infra_files=[],
        mod_name='item',
        op_level=2,
        perm_desc='Manipulates items in inventories',
        commit_msg='feat: implement /item command',
        pr_title='feat: implement /item command',
        pr_body='## Summary\\n- Implements `/item` command for manipulating items in entity and block inventories\\n- Uses vanilla translation keys\\n\\n## Test plan\\n- [ ] `/item replace entity @s weapon.mainhand with diamond_sword` - set item\\n- [ ] `/item modify entity @s weapon.mainhand enchant_randomly` - modify item',
    )

    # Translation fixes (separate atomic PR)
    print(f"\n{'='*60}")
    print("Creating: fix/translation-keys")
    print(f"{'='*60}")

    run('git branch -D fix/translation-keys', check=False)
    run('git checkout -b fix/translation-keys origin/master')
    run('git checkout feat/cmd-execute-attribute-item -- pumpkin/src/command/commands/fill.rs')
    run('git checkout feat/cmd-execute-attribute-item -- pumpkin/src/command/commands/setworldspawn.rs')
    run('git checkout feat/cmd-execute-attribute-item -- pumpkin/src/command/commands/spawnpoint.rs')
    run('git checkout feat/cmd-execute-attribute-item -- pumpkin/src/command/commands/teleport.rs')
    run('git add -A')
    run('git commit -m "fix: use translation constants instead of string literals\\n\\nReplace hardcoded translation key strings with proper constants\\nfrom pumpkin_data::translation in fill, setworldspawn, spawnpoint,\\nand teleport commands."')
    run('git push fork fix/translation-keys --force')
    pr_result = run(
        'gh pr create --repo Pumpkin-MC/Pumpkin --head WhiteProject1:fix/translation-keys --base master '
        '--title "fix: use translation constants instead of string literals" '
        '--body "## Summary\\n- Replace hardcoded translation key strings with `pumpkin_data::translation` constants\\n- Affected commands: fill, setworldspawn, spawnpoint, teleport\\n\\n## Test plan\\n- [ ] `cargo clippy -p pumpkin --all-targets --all-features -- -Dwarnings`\\n- [ ] Verify affected commands still work correctly"',
        check=False
    )
    if pr_result:
        print(f"PR created: {pr_result}")

    # Return to master
    run('git checkout origin/master --detach', check=False)

    print("\n" + "="*60)
    print("DONE! All atomic branches created and pushed.")
    print("="*60)


if __name__ == '__main__':
    main()
