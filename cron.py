"""
cron.py — Cron Dispatch Reporter + Agent Poll Loop

Three modes:
  1. `python cron.py status`   — Report which agents have pending broadcasts
  2. `python cron.py poll`     — Agent poll loop: check broadcasts, hibernate, repeat
  3. `python cron.py send`     — Send a broadcast to specific agents

Architecture:
  - Architect broadcasts to specific agents via bb.broadcast()
  - Each agent session calls bb.hydrate() which auto-checks broadcasts (DI)
  - Between polls, agents hibernate (sleep) until the next check interval
  - This script provides the poll loop that agents run inside Claude Code

Crontab example (status report every 5 min):
    */5 * * * * cd /home/user/Pumpkin && python cron.py status >> /tmp/cron.log 2>&1

Agent poll (run inside a Claude Code session or terminal):
    python cron.py poll --agent entity --interval 300
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
    print(f"[crown {ts}] {msg}", flush=True)


async def get_watermarks(redis: RedisClient) -> dict[str, float]:
    """Read all agent watermarks from Redis hash."""
    key = f"ada:crown:{PROJECT}:watermarks"
    raw = await redis.cmd("HGETALL", key)
    if not raw:
        return {}
    wm = {}
    for i in range(0, len(raw), 2):
        wm[raw[i]] = float(raw[i + 1])
    return wm


async def update_watermark(redis: RedisClient, agent: str, ts: float):
    """Advance an agent's watermark after processing broadcasts."""
    key = f"ada:crown:{PROJECT}:watermarks"
    await redis.cmd("HSET", key, agent, str(ts))


async def check_agent_broadcasts(redis: RedisClient, agent: str, since_ts: float) -> list[dict]:
    """Get broadcasts for an agent newer than since_ts."""
    key = f"ada:broadcast:{PROJECT}:{agent}"
    min_score = f"({since_ts}" if since_ts > 0 else "-inf"
    results = await redis.cmd("ZRANGEBYSCORE", key, min_score, "+inf")
    return [json.loads(r) for r in (results or [])]


# ── Mode 1: Status Report ────────────────────────────────────────

async def status():
    """Check all agents for pending broadcasts. Print report."""
    redis = RedisClient()
    try:
        watermarks = await get_watermarks(redis)
        log("Agent broadcast status:")

        total_pending = 0
        for agent in AGENTS:
            since = watermarks.get(agent, 0.0)
            broadcasts = await check_agent_broadcasts(redis, agent, since)
            count = len(broadcasts)
            total_pending += count
            if count > 0:
                subjects = ", ".join(b.get("subject", "?")[:40] for b in broadcasts[:3])
                log(f"  {agent:12s}: {count} pending  [{subjects}]")
            else:
                log(f"  {agent:12s}: idle")

        log(f"Total pending: {total_pending}")
        return total_pending
    finally:
        await redis.close()


# ── Mode 2: Agent Poll Loop ──────────────────────────────────────

async def poll(agent: str, interval: int = 300):
    """
    Poll loop for a single agent. Checks broadcasts, prints them,
    advances watermark, then sleeps.

    Run this inside a Claude Code session or terminal:
        python crown.py poll --agent entity --interval 300

    The output tells the Claude Code session what work is pending.
    The session can then process it and call poll again.
    """
    redis = RedisClient()
    try:
        watermarks = await get_watermarks(redis)
        since = watermarks.get(agent, 0.0)

        while True:
            broadcasts = await check_agent_broadcasts(redis, agent, since)

            if broadcasts:
                log(f"[{agent}] {len(broadcasts)} broadcast(s) received:")
                for b in broadcasts:
                    from_agent = b.get("from_agent", "?")
                    subject = b.get("subject", "?")
                    btype = b.get("type", "?")
                    priority = b.get("priority", "normal")
                    log(f"  [{priority}] {btype} from {from_agent}: {subject}")
                    body = b.get("body")
                    if body:
                        log(f"    body: {json.dumps(body, default=str)[:200]}")

                # Print full JSON for machine consumption
                print(f"\n--- BROADCASTS_JSON ---")
                print(json.dumps(broadcasts, indent=2, default=str))
                print(f"--- END_BROADCASTS ---\n")

                # Advance watermark
                since = max(b.get("ts_score", 0.0) for b in broadcasts)
                await update_watermark(redis, agent, since)

                # Also check inbox for handovers
                bb = Blackboard(PROJECT, agent_id=agent)
                bb.redis = redis  # Reuse connection
                handover = await bb.receive_handover()
                if handover:
                    log(f"[{agent}] Handover from {handover.get('from_agent', '?')}: "
                        f"{handover.get('request', {}).get('task', '?')}")

                # Exit after finding work — let Claude Code process it
                # The next cron cycle or manual poll will check again
                return broadcasts

            else:
                log(f"[{agent}] No broadcasts. Hibernating {interval}s...")
                await asyncio.sleep(interval)

                # Refresh watermarks in case another session updated them
                watermarks = await get_watermarks(redis)
                since = max(since, watermarks.get(agent, 0.0))

    except KeyboardInterrupt:
        log(f"[{agent}] Poll interrupted.")
    finally:
        await redis.close()


# ── Mode 3: Broadcast (for Architect to use from CLI) ─────────────

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


# ── CLI Entry Point ──────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Crown — Agent Broadcast Dispatcher")
    sub = parser.add_subparsers(dest="mode")

    # Status
    sub.add_parser("status", help="Report pending broadcasts for all agents")

    # Poll
    p_poll = sub.add_parser("poll", help="Poll loop for a single agent")
    p_poll.add_argument("--agent", required=True, help="Agent name")
    p_poll.add_argument("--interval", type=int, default=300, help="Sleep interval in seconds")

    # Send
    p_send = sub.add_parser("send", help="Send a broadcast")
    p_send.add_argument("--from", dest="from_agent", default="architect")
    p_send.add_argument("--to", required=True, help="Comma-separated agent names, or 'all'")
    p_send.add_argument("--type", dest="btype", default="task")
    p_send.add_argument("--subject", required=True)
    p_send.add_argument("--body", default="{}")

    args = parser.parse_args()

    if args.mode == "status":
        asyncio.run(status())
    elif args.mode == "poll":
        asyncio.run(poll(args.agent, args.interval))
    elif args.mode == "send":
        to_agents = AGENTS if args.to == "all" else args.to.split(",")
        asyncio.run(send_broadcast(args.from_agent, to_agents,
                                    args.btype, args.subject, args.body))
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
