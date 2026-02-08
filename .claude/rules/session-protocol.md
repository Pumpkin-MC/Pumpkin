# SESSION PROTOCOL — MANDATORY

You are an agent in the Pumpkin project (AdaWorldAPI/Pumpkin fork). You have ONE job, ONE scope,
and ONE set of rules. Follow them exactly.

## Your Identity

Read `.current-agent` to know which agent you are.
Read `.claude/contracts/{your-agent}.toml` for your boundaries.
Read `.claude/ORCHESTRATOR.md` if you need the full constitution.

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
- NEVER modify `pumpkin-util/` unless you are the Architect
- NEVER modify another agent's folder
- USE `.claude/reference/` for inspiration but do not copy blindly — rewrite in your own style
- RUN your tests before finishing: check `must_pass` in your contract
- REFERENCE specs in `.claude/specs/` as your source of truth, not guesses

## Before Finishing

1. Write your session log: `.claude/sessions/{today}/{seq}_{agent}_{description}.md`
2. Follow the log format in .claude/ORCHESTRATOR.md exactly — every section matters
3. Update `.claude/sessions/decisions/{agent}.md` if you made any decisions
4. Ensure all tests pass
5. Commit with message: `[{agent}] {description}`

## Shared Interface Requests

If you need a new type or trait in `pumpkin-util/`:
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
2. **STAY IN YOUR LANE** — your folder, your tests, your logs
3. **LOG EVERYTHING** — no session exists without a log entry
4. **TEST YOUR WORK** — broken tests = invalid session
5. **DECISIONS ARE APPEND-ONLY** — never delete, only supersede with rationale
6. **SHARED TYPES ARE SACRED** — only Architect touches pumpkin-util/
7. **SPECS ARE TRUTH** — when in doubt, the spec wins over Pumpkin's interpretation
