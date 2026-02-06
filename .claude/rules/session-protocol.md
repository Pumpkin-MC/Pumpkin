# SESSION PROTOCOL — MANDATORY

You are an agent in the AdaWorldAPI/Pumpkin project (a fork of Pumpkin-MC/Pumpkin).
You have ONE job, ONE set of crates, and ONE set of rules. Follow them exactly.

## Your Identity

Read `.current-agent` to know which agent you are.
Read `contracts/{your-agent}.toml` for your boundaries.
Read `ORCHESTRATOR.md` if you need the full constitution.

## Before Writing Any Code

You MUST read, in this order:
1. Every file in `.claude/sessions/{today}/`
2. The last 5 files in `.claude/sessions/{yesterday}/`
3. `.claude/sessions/decisions/{your-agent}.md`
4. `.claude/sessions/decisions/architect.md`
5. Any log file from any day that mentions your agent name

Then write your session preamble in your log file FIRST, proving you read these.
Only then may you begin coding.

If there are no logs yet (you are the first session), state that explicitly in your preamble.

## While Working

- Write ONLY to paths listed in your contract's write_paths
- Write ONLY to `.claude/sessions/` (always allowed)
- NEVER modify shared crates (pumpkin-util, pumpkin-data) unless you are the Architect
- NEVER modify another agent's crate or module
- This is a living codebase — EXTEND existing code, don't rewrite without good reason
- wiki.vg spec is truth; existing Pumpkin code is guidance
- RUN your tests before finishing: check `must_pass` in your contract

## Before Finishing

1. Write your session log: `.claude/sessions/{today}/{seq}_{agent}_{description}.md`
2. Follow the log format in ORCHESTRATOR.md exactly — every section matters
3. Update `.claude/sessions/decisions/{agent}.md` if you made any decisions
4. Ensure all tests pass
5. Commit with message: `[{agent}] {description}`

## Shared Crate Requests

If you need a new type or trait in `pumpkin-util/` or `pumpkin-data/`:
- Do NOT create it yourself (unless you are Architect)
- Document what you need in "What I Need From Others → Architect"
- Propose the exact signature you want
- Continue your work using a local placeholder or TODO if needed

## How to Read Logs

When reading other agents' logs, pay special attention to:
- **⚠️ markers** — these indicate shared type/trait changes
- **"What I Need From Others"** — check if anyone needs something from you
- **"What Others Should Know"** — conventions that may affect your work
- **"Decisions Made"** — rulings that constrain your options

If someone requested something from you, acknowledge it in your preamble and address it.

## Non-Negotiable Rules

1. **READ BEFORE WRITE** — always, no exceptions, prove it in preamble
2. **STAY IN YOUR LANE** — your crates, your modules, your logs
3. **LOG EVERYTHING** — no session exists without a log entry
4. **TEST YOUR WORK** — broken tests = invalid session
5. **DECISIONS ARE APPEND-ONLY** — never delete, only supersede with rationale
6. **SHARED CRATES ARE SACRED** — only Architect touches pumpkin-util/ and pumpkin-data/
7. **SPEC IS TRUTH** — when in doubt, wiki.vg wins over Pumpkin's interpretation
8. **EXTEND, DON'T REWRITE** — this is a fork, not greenfield
