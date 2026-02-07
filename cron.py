"""
cron.py — Agent Orchestrator + Dispatch + Poll

Modes:
  1. `python cron.py status`      — Broadcast + task board status for all agents
  2. `python cron.py poll`        — Agent poll loop: hibernate until work arrives
  3. `python cron.py send`        — Send a broadcast to specific agents
  4. `python cron.py dispatch`    — Orchestrator dispatches a task to an agent
  5. `python cron.py board`       — Show the full task board
  6. `python cron.py plan`        — Dispatch a multi-agent plan from JSON file

Architecture:
  - Orchestrator (Architect) dispatches tasks via bb.dispatch_task()
  - Each agent session calls bb.hydrate() which auto-checks broadcasts + tasks (DI)
  - Between polls, agents hibernate (sleep) until the next check interval
  - Agents claim tasks, do work, then complete_task() or fail_task()
  - CI alternative: GitHub Actions workflow_dispatch triggered by task post

Usage:
    python cron.py status
    python cron.py poll --agent entity --interval 300
    python cron.py dispatch --to entity --task "Implement EntitySpawnEvent"
    python cron.py board
    python cron.py plan --file sprint_plan.json
"""

import asyncio
import json
import os
import sys
from datetime import datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from blackboard import Blackboard, RedisClient, REDIS_URL, REDIS_TOKEN, _ts

PROJECT = "pumpkin"

AGENTS = [
    "architect", "core", "protocol", "worldgen",
    "entity", "items", "redstone", "storage", "plugin",
]


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[cron {ts}] {msg}", flush=True)


async def get_watermarks(redis: RedisClient) -> dict[str, float]:
    """Read all agent watermarks from Redis hash."""
    key = f"ada:cron:{PROJECT}:watermarks"
    raw = await redis.cmd("HGETALL", key)
    if not raw:
        return {}
    wm = {}
    for i in range(0, len(raw), 2):
        wm[raw[i]] = float(raw[i + 1])
    return wm


async def update_watermark(redis: RedisClient, agent: str, ts: float):
    """Advance an agent's watermark after processing broadcasts."""
    key = f"ada:cron:{PROJECT}:watermarks"
    await redis.cmd("HSET", key, agent, str(ts))


async def check_agent_broadcasts(redis: RedisClient, agent: str, since_ts: float) -> list[dict]:
    """Get broadcasts for an agent newer than since_ts."""
    key = f"ada:broadcast:{PROJECT}:{agent}"
    min_score = f"({since_ts}" if since_ts > 0 else "-inf"
    results = await redis.cmd("ZRANGEBYSCORE", key, min_score, "+inf")
    return [json.loads(r) for r in (results or [])]


# ── Mode 1: Status Report ────────────────────────────────────────

async def status():
    """Check all agents for pending broadcasts + tasks. Print report."""
    bb = Blackboard(PROJECT, agent_id="cron")
    try:
        redis = bb.redis
        watermarks = await get_watermarks(redis)

        log("=== Broadcast Status ===")
        total_bc = 0
        for agent in AGENTS:
            since = watermarks.get(agent, 0.0)
            broadcasts = await check_agent_broadcasts(redis, agent, since)
            count = len(broadcasts)
            total_bc += count
            if count > 0:
                subjects = ", ".join(b.get("subject", "?")[:40] for b in broadcasts[:3])
                log(f"  {agent:12s}: {count} pending  [{subjects}]")
            else:
                log(f"  {agent:12s}: idle")
        log(f"Total broadcasts pending: {total_bc}")

        log("")
        log("=== Task Board ===")
        board = await bb.get_task_board()
        if not board:
            log("  No tasks dispatched")
        else:
            by_status = {}
            for tid, info in board.items():
                s = info.get("status", "?")
                by_status.setdefault(s, []).append((tid, info))

            for s in ["dispatched", "claimed", "done", "failed"]:
                tasks = by_status.get(s, [])
                if tasks:
                    log(f"  [{s.upper()}] ({len(tasks)})")
                    for tid, info in tasks:
                        agent = info.get("agent", "?")
                        task = info.get("task", "?")[:50]
                        log(f"    {tid[:16]:16s}  {agent:12s}  {task}")

        log("")
        log("=== Agent Registry ===")
        agents_raw = await redis.cmd("HGETALL", f"ada:bb:{PROJECT}:agents")
        if agents_raw:
            for i in range(0, len(agents_raw), 2):
                log(f"  {agents_raw[i]:12s}: {agents_raw[i+1]}")

        return total_bc
    finally:
        await bb.close()


# ── Mode 2: Agent Poll Loop ──────────────────────────────────────

async def poll(agent: str, interval: int = 300):
    """
    Poll loop for a single agent. Checks broadcasts + tasks.
    Hibernates between checks. Exits when work is found.

    Run inside a Claude Code session:
        python cron.py poll --agent entity --interval 300
    """
    redis = RedisClient()
    try:
        watermarks = await get_watermarks(redis)
        since = watermarks.get(agent, 0.0)

        while True:
            # Check broadcasts
            broadcasts = await check_agent_broadcasts(redis, agent, since)

            # Check task queue
            task_queue = await redis.cmd(
                "LRANGE", f"ada:tasks:{PROJECT}:queue:{agent}", "0", "-1"
            )

            has_work = bool(broadcasts) or bool(task_queue)

            if has_work:
                if broadcasts:
                    log(f"[{agent}] {len(broadcasts)} broadcast(s):")
                    for b in broadcasts:
                        log(f"  [{b.get('priority', 'normal')}] "
                            f"{b.get('type', '?')} from {b.get('from_agent', '?')}: "
                            f"{b.get('subject', '?')}")

                    print(f"\n--- BROADCASTS_JSON ---")
                    print(json.dumps(broadcasts, indent=2, default=str))
                    print(f"--- END_BROADCASTS ---\n")

                    since = max(b.get("ts_score", 0.0) for b in broadcasts)
                    await update_watermark(redis, agent, since)

                if task_queue:
                    log(f"[{agent}] {len(task_queue)} task(s) in queue:")
                    for tid in task_queue:
                        record = await redis.get_json(f"ada:tasks:{PROJECT}:{tid}")
                        if record:
                            log(f"  [{record.get('priority', 'normal')}] "
                                f"{tid}: {record.get('task', '?')}")

                    print(f"\n--- TASKS_JSON ---")
                    tasks_detail = []
                    for tid in task_queue:
                        record = await redis.get_json(f"ada:tasks:{PROJECT}:{tid}")
                        if record:
                            tasks_detail.append(record)
                    print(json.dumps(tasks_detail, indent=2, default=str))
                    print(f"--- END_TASKS ---\n")

                return {"broadcasts": broadcasts or [], "tasks": task_queue or []}

            else:
                log(f"[{agent}] No work. Hibernating {interval}s...")
                await asyncio.sleep(interval)

                watermarks = await get_watermarks(redis)
                since = max(since, watermarks.get(agent, 0.0))

    except KeyboardInterrupt:
        log(f"[{agent}] Poll interrupted.")
    finally:
        await redis.close()


# ── Mode 3: Send Broadcast ───────────────────────────────────────

async def send_broadcast(from_agent: str, to_agents: list[str],
                         btype: str, subject: str, body: str):
    """Send a broadcast from CLI."""
    bb = Blackboard(PROJECT, agent_id=from_agent)
    try:
        message = {
            "type": btype,
            "subject": subject,
            "body": json.loads(body) if body.startswith("{") else {"text": body},
            "priority": "normal",
        }
        bid = await bb.broadcast(to_agents, message)
        log(f"Broadcast {bid} sent to {to_agents}")
        return bid
    finally:
        await bb.close()


# ── Mode 4: Dispatch Task ────────────────────────────────────────

async def dispatch(from_agent: str, to_agent: str, task: str,
                   description: str, priority: str, context_str: str):
    """Dispatch a task from CLI (orchestrator role)."""
    bb = Blackboard(PROJECT, agent_id=from_agent)
    try:
        ctx = json.loads(context_str) if context_str.startswith("{") else None
        tid = await bb.dispatch_task(
            to_agent=to_agent,
            task=task,
            description=description,
            context=ctx,
            priority=priority,
        )
        log(f"Task {tid} dispatched to {to_agent}: {task}")
        return tid
    finally:
        await bb.close()


# ── Mode 5: Task Board ───────────────────────────────────────────

async def board():
    """Show the full task board."""
    bb = Blackboard(PROJECT, agent_id="cron")
    try:
        task_board = await bb.get_task_board()
        if not task_board:
            log("Task board is empty")
            return

        log(f"Task board ({len(task_board)} tasks):")
        for tid, info in sorted(task_board.items(),
                                 key=lambda x: x[1].get("dispatched_at", "")):
            status = info.get("status", "?")
            agent = info.get("agent", "?")
            task = info.get("task", "?")
            priority = info.get("priority", "normal")
            marker = {"dispatched": ".", "claimed": ">", "done": "+", "failed": "X"}.get(status, "?")
            log(f"  {marker} [{priority:6s}] {agent:12s} {status:12s} {task[:60]}")

        # Print JSON for machine consumption
        print(f"\n--- BOARD_JSON ---")
        print(json.dumps(task_board, indent=2, default=str))
        print(f"--- END_BOARD ---\n")
    finally:
        await bb.close()


# ── Mode 6: Dispatch Plan ────────────────────────────────────────

async def dispatch_plan(from_agent: str, plan_file: str):
    """
    Dispatch a multi-agent plan from a JSON file.

    Plan format:
    [
        {"agent": "entity", "task": "Implement EntitySpawnEvent",
         "description": "...", "priority": "high"},
        {"agent": "redstone", "task": "Add piston head block state"},
        ...
    ]
    """
    bb = Blackboard(PROJECT, agent_id=from_agent)
    try:
        with open(plan_file) as f:
            plan = json.load(f)

        log(f"Dispatching plan with {len(plan)} tasks from {plan_file}")
        task_ids = await bb.dispatch_plan(plan)

        for tid, item in zip(task_ids, plan):
            log(f"  {tid} → {item['agent']}: {item['task']}")

        log(f"Plan dispatched: {len(task_ids)} tasks")
        return task_ids
    finally:
        await bb.close()


# ── CLI Entry Point ──────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(
        description="cron.py — Agent Orchestrator + Dispatch + Poll"
    )
    sub = parser.add_subparsers(dest="mode")

    # Status
    sub.add_parser("status", help="Broadcast + task board status for all agents")

    # Poll
    p_poll = sub.add_parser("poll", help="Agent poll loop (hibernate until work)")
    p_poll.add_argument("--agent", required=True, help="Agent name")
    p_poll.add_argument("--interval", type=int, default=300,
                        help="Sleep interval in seconds (default 300)")

    # Send broadcast
    p_send = sub.add_parser("send", help="Send a broadcast to agents")
    p_send.add_argument("--from", dest="from_agent", default="architect")
    p_send.add_argument("--to", required=True,
                        help="Comma-separated agent names, or 'all'")
    p_send.add_argument("--type", dest="btype", default="task")
    p_send.add_argument("--subject", required=True)
    p_send.add_argument("--body", default="{}")

    # Dispatch task
    p_dispatch = sub.add_parser("dispatch", help="Dispatch a task to an agent")
    p_dispatch.add_argument("--from", dest="from_agent", default="architect")
    p_dispatch.add_argument("--to", required=True, help="Target agent")
    p_dispatch.add_argument("--task", required=True, help="Task name")
    p_dispatch.add_argument("--description", default="", help="Task description")
    p_dispatch.add_argument("--priority", default="normal",
                            choices=["high", "normal", "low"])
    p_dispatch.add_argument("--context", default="{}", help="JSON context")

    # Board
    sub.add_parser("board", help="Show full task board")

    # Plan
    p_plan = sub.add_parser("plan", help="Dispatch a plan from JSON file")
    p_plan.add_argument("--from", dest="from_agent", default="architect")
    p_plan.add_argument("--file", required=True, help="Path to plan JSON file")

    args = parser.parse_args()

    if args.mode == "status":
        asyncio.run(status())
    elif args.mode == "poll":
        asyncio.run(poll(args.agent, args.interval))
    elif args.mode == "send":
        to_agents = AGENTS if args.to == "all" else args.to.split(",")
        asyncio.run(send_broadcast(args.from_agent, to_agents,
                                    args.btype, args.subject, args.body))
    elif args.mode == "dispatch":
        asyncio.run(dispatch(args.from_agent, args.to, args.task,
                              args.description, args.priority, args.context))
    elif args.mode == "board":
        asyncio.run(board())
    elif args.mode == "plan":
        asyncio.run(dispatch_plan(args.from_agent, args.file))
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
