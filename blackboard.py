"""
blackboard.py — Upstash Redis Session↔Session / Agent↔Agent Blackboard

Drop this into any project. Claude Code uses it as the persistence layer
for cross-session state, agent handovers, and ice-caked decisions.

Usage:
    from blackboard import Blackboard

    bb = Blackboard("my-project")
    state = await bb.hydrate()          # SESSION START
    ...work...
    await bb.ice_cake(decision)         # MID-SESSION
    await bb.persist(state)             # SESSION END
"""

import asyncio
import json
import os
import uuid
from datetime import datetime, timezone
from typing import Any

try:
    import httpx
except ImportError:
    import subprocess
    subprocess.check_call(["pip", "install", "httpx", "--break-system-packages", "-q"])
    import httpx


# --- Config ---
# All credentials MUST come from environment variables. No hardcoded fallbacks.

def _require_env(name: str) -> str:
    val = os.environ.get(name)
    if not val:
        raise EnvironmentError(f"Required environment variable {name} is not set")
    # Strip stray quotes that some shell configs embed
    return val.strip('"').strip("'")

REDIS_URL = _require_env("UPSTASH_REDIS_REST_URL")
REDIS_TOKEN = _require_env("UPSTASH_REDIS_REST_TOKEN")

# Secondary Redis for hot cache / RAM expansion
REDIS_URL_2 = _require_env("UPSTASH_REDIS_REST_URL2")
REDIS_TOKEN_2 = _require_env("UPSTASH_REDIS_REST_TOKEN2")


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _ts() -> float:
    return datetime.now(timezone.utc).timestamp()


def _uid() -> str:
    return uuid.uuid4().hex[:12]


class RedisClient:
    """Thin async wrapper around Upstash REST API."""

    def __init__(self, url: str = REDIS_URL, token: str = REDIS_TOKEN):
        self.url = url.rstrip("/")
        self.headers = {"Authorization": f"Bearer {token}"}
        self._client = httpx.AsyncClient(timeout=10)

    async def cmd(self, *args) -> Any:
        """Execute a single Redis command via REST (POST for reliability with large payloads)."""
        resp = await self._client.post(
            self.url,
            headers=self.headers,
            json=list(args),
        )
        resp.raise_for_status()
        return resp.json().get("result")

    async def pipeline(self, commands: list[list]) -> list:
        """Execute multiple commands atomically."""
        resp = await self._client.post(
            f"{self.url}/pipeline",
            headers=self.headers,
            json=commands,
        )
        resp.raise_for_status()
        return resp.json()

    async def get_json(self, key: str) -> dict | None:
        raw = await self.cmd("GET", key)
        return json.loads(raw) if raw else None

    async def set_json(self, key: str, value: dict, ex: int | None = None):
        payload = json.dumps(value, default=str)
        if ex:
            await self.cmd("SET", key, payload, "EX", ex)
        else:
            await self.cmd("SET", key, payload)

    async def close(self):
        await self._client.aclose()


class Blackboard:
    """
    Session↔Session / Agent↔Agent orchestration blackboard.

    Key schema:
        ada:bb:{project}:state           — Current state
        ada:bb:{project}:decisions       — Sorted set of decisions
        ada:bb:{project}:handover:{id}   — Handover packets
        ada:bb:{project}:ice_cake        — Layer counter
        ada:bb:{project}:agents          — Agent registry
        ada:bb:{project}:log             — Session log (list)
        ada:a2a:inbox:{agent_id}         — Agent inbox
        ada:session:{session_id}         — Session snapshot
        ada:session:latest:{project}     — Latest session pointer
        ada:broadcast:{project}:{agent}  — Per-agent broadcast channel (sorted set)
        ada:broadcast:{project}:ack:{id} — Broadcast acknowledgements (hash)
    """

    def __init__(self, project: str, agent_id: str = "orchestrator"):
        self.project = project
        self.agent_id = agent_id
        self.session_id = f"sess_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}_{_uid()}"
        self.redis = RedisClient()
        self.redis_hot = RedisClient(REDIS_URL_2, REDIS_TOKEN_2)
        self._state: dict | None = None

    # ── Session Lifecycle ──────────────────────────────────────

    async def hydrate(self) -> dict:
        """MANDATORY first call. Read blackboard from Redis."""
        state = await self.redis.get_json(f"ada:bb:{self.project}:state")

        if state:
            # Carry forward previous session info
            prev = {
                "session_id": state.get("session_id"),
                "summary": state.get("summary", ""),
                "ice_cake_layers": state.get("ice_cake_layers", 0),
            }
            previous = state.get("previous_sessions", [])
            previous.insert(0, prev)
            state["previous_sessions"] = previous[:5]  # Keep last 5
            state["session_id"] = self.session_id
            state["last_updated"] = _now()
            state["status"] = "HYDRATED"
        else:
            state = self._fresh_state()

        self._state = state

        # Check inbox for pending handovers
        handovers = await self._check_inbox()
        if handovers:
            state["pending_handovers"] = handovers

        # DI: Check broadcasts during hydration
        last_ts = state.get("last_broadcast_seen", 0.0)
        broadcasts = await self.check_broadcasts(since_ts=last_ts)
        if broadcasts:
            state["pending_broadcasts"] = broadcasts
            # Advance watermark to newest broadcast
            state["last_broadcast_seen"] = max(
                b.get("ts_score", 0.0) for b in broadcasts
            )

        return state

    async def persist(self, state: dict | None = None):
        """MANDATORY last call. Write blackboard to Redis."""
        state = state or self._state
        if not state:
            raise RuntimeError("No state to persist. Did you forget to hydrate()?")

        state["last_updated"] = _now()
        state["status"] = "PERSISTED"

        cmds = [
            ["SET", f"ada:bb:{self.project}:state", json.dumps(state, default=str)],
            ["SET", f"ada:session:latest:{self.project}", self.session_id],
            ["SET", f"ada:session:{self.session_id}", json.dumps(state, default=str)],
            ["LPUSH", f"ada:bb:{self.project}:log", json.dumps({
                "session_id": self.session_id,
                "agent": self.agent_id,
                "timestamp": _now(),
                "summary": state.get("summary", ""),
                "decisions_count": len(state.get("decisions", [])),
                "ice_cake_layers": state.get("ice_cake_layers", 0),
            }, default=str)],
            # Trim log to last 100 entries
            ["LTRIM", f"ada:bb:{self.project}:log", "0", "99"],
        ]

        await self.redis.pipeline(cmds)

    # ── Ice Caking ─────────────────────────────────────────────

    async def ice_cake(self, decision: dict):
        """
        Ice cake a FLOW decision immediately to Redis.

        decision = {
            "task": "Use JWT over session cookies",
            "rationale": "Stateless, works with API",
            "gate": "FLOW",
            "gate_sd": 0.08,
        }
        """
        decision["ice_caked"] = True
        decision["timestamp"] = _now()
        decision["session_id"] = self.session_id
        decision["agent"] = self.agent_id

        ts = _ts()
        cmds = [
            ["ZADD", f"ada:bb:{self.project}:decisions", str(ts), json.dumps(decision, default=str)],
            ["INCR", f"ada:bb:{self.project}:ice_cake"],
        ]
        await self.redis.pipeline(cmds)

        # Also update local state
        if self._state:
            self._state.setdefault("decisions", []).append(decision)
            self._state["ice_cake_layers"] = self._state.get("ice_cake_layers", 0) + 1

    async def get_ice_cake_count(self) -> int:
        result = await self.redis.cmd("GET", f"ada:bb:{self.project}:ice_cake")
        return int(result) if result else 0

    async def get_recent_decisions(self, n: int = 10) -> list[dict]:
        """Get last N ice-caked decisions."""
        results = await self.redis.cmd(
            "ZREVRANGE", f"ada:bb:{self.project}:decisions", "0", str(n - 1)
        )
        return [json.loads(r) for r in (results or [])]

    # ── Agent-to-Agent ─────────────────────────────────────────

    async def post_handover(self, to_agent: str, task: str,
                            context: dict | None = None,
                            expected_output: str = "",
                            constraints: list[str] | None = None) -> str:
        """Post a handover packet for a target agent."""
        hid = f"ho_{_uid()}"
        callback_key = f"ada:a2a:result:{hid}"

        handover = {
            "id": hid,
            "from_agent": self.agent_id,
            "to_agent": to_agent,
            "session_id": self.session_id,
            "project": self.project,
            "timestamp": _now(),
            "context": context or {
                "current_task": self._state.get("current_task") if self._state else None,
                "decisions_made": (self._state.get("decisions", [])[-5:]) if self._state else [],
                "files_touched": self._state.get("files_touched", {}) if self._state else {},
                "blockers": self._state.get("blockers", []) if self._state else [],
            },
            "request": {
                "task": task,
                "expected_output": expected_output,
                "constraints": constraints or [],
            },
            "return_to": {
                "agent": self.agent_id,
                "callback_key": callback_key,
            },
        }

        cmds = [
            ["SET", f"ada:bb:{self.project}:handover:{hid}", json.dumps(handover, default=str)],
            ["LPUSH", f"ada:a2a:inbox:{to_agent}", hid],
            ["HSET", f"ada:bb:{self.project}:agents", to_agent, "spawning"],
        ]
        await self.redis.pipeline(cmds)

        return hid

    async def receive_handover(self) -> dict | None:
        """Check inbox for pending handover. Called at agent start."""
        hid = await self.redis.cmd("RPOP", f"ada:a2a:inbox:{self.agent_id}")
        if not hid:
            return None
        return await self.redis.get_json(f"ada:bb:{self.project}:handover:{hid}")

    async def post_result(self, handover: dict, result: dict):
        """Post result back to the spawning agent."""
        callback_key = handover["return_to"]["callback_key"]
        parent_agent = handover["return_to"]["agent"]

        cmds = [
            ["SET", callback_key, json.dumps(result, default=str)],
            ["LPUSH", f"ada:a2a:inbox:{parent_agent}", json.dumps({
                "type": "result",
                "handover_id": handover["id"],
                "key": callback_key,
            })],
            ["HSET", f"ada:bb:{self.project}:agents", self.agent_id, "completed"],
        ]
        await self.redis.pipeline(cmds)

    async def check_result(self, handover_id: str) -> dict | None:
        """Check if a spawned agent has posted results."""
        return await self.redis.get_json(f"ada:a2a:result:{handover_id}")

    # ── Collapse Gate ──────────────────────────────────────────

    @staticmethod
    def collapse_gate(scores: list[float]) -> tuple[str, float]:
        """
        Triangle collapse: top 3 scores → gate decision.
        Returns (gate, sd).
        """
        top3 = sorted(scores, reverse=True)[:3]
        mean = sum(top3) / len(top3)
        variance = sum((x - mean) ** 2 for x in top3) / len(top3)
        sd = variance ** 0.5

        if sd < 0.15:
            return "FLOW", sd
        elif sd > 0.35:
            return "BLOCK", sd
        else:
            return "HOLD", sd

    # ── Broadcast (Agent-Scoped) ────────────────────────────────

    async def broadcast(self, to_agents: list[str], message: dict):
        """
        Send a broadcast to specific agents.

        Each agent has its own broadcast channel:
            ada:broadcast:{project}:{agent_id}

        message = {
            "type": "task" | "decision" | "unblock" | "status_request",
            "subject": "Short description",
            "body": { ... },
            "priority": "high" | "normal" | "low",
        }
        """
        ts = _ts()
        envelope = {
            "id": f"bc_{_uid()}",
            "from_agent": self.agent_id,
            "timestamp": _now(),
            "ts_score": ts,
            **message,
        }
        payload = json.dumps(envelope, default=str)

        cmds = []
        for agent in to_agents:
            key = f"ada:broadcast:{self.project}:{agent}"
            cmds.append(["ZADD", key, str(ts), payload])
            # Keep only last 50 broadcasts per agent to prevent unbounded growth
            cmds.append(["ZREMRANGEBYRANK", key, "0", "-51"])

        await self.redis.pipeline(cmds)
        return envelope["id"]

    async def broadcast_all(self, message: dict, exclude: list[str] | None = None):
        """
        Broadcast to all registered agents (except excluded ones).
        Reads the agent registry to get the list.
        """
        agents_raw = await self.redis.cmd("HKEYS", f"ada:bb:{self.project}:agents")
        agents = [a for a in (agents_raw or []) if a not in (exclude or [])]
        if agents:
            return await self.broadcast(agents, message)
        return None

    async def check_broadcasts(self, since_ts: float = 0.0) -> list[dict]:
        """
        Check for broadcasts sent to this agent since a given timestamp.

        Args:
            since_ts: Unix timestamp. Only broadcasts after this are returned.
                      Pass 0.0 to get all. Typical usage: pass the ts from
                      the last broadcast you processed.

        Returns:
            List of broadcast envelopes, oldest first.
        """
        key = f"ada:broadcast:{self.project}:{self.agent_id}"
        # ZRANGEBYSCORE key min max — exclusive min with "("
        min_score = f"({since_ts}" if since_ts > 0 else "-inf"
        results = await self.redis.cmd("ZRANGEBYSCORE", key, min_score, "+inf")
        return [json.loads(r) for r in (results or [])]

    async def ack_broadcast(self, broadcast_id: str, response: dict | None = None):
        """
        Acknowledge a broadcast, optionally with a response.
        Stores the ack so the sender can check status.
        """
        ack = {
            "broadcast_id": broadcast_id,
            "agent": self.agent_id,
            "timestamp": _now(),
            "response": response,
        }
        key = f"ada:broadcast:{self.project}:ack:{broadcast_id}"
        await self.redis.cmd(
            "HSET", key, self.agent_id, json.dumps(ack, default=str)
        )

    async def wait_for_broadcast(self, poll_interval: int = 300,
                                  timeout: int = 0) -> list[dict]:
        """
        Block until broadcasts arrive for this agent, then return them.

        Usage inside a Claude Code session:
            broadcasts = await bb.wait_for_broadcast()
            # process broadcasts...
            # call again to wait for more

        Args:
            poll_interval: Seconds between polls (default 300 = 5 min)
            timeout: Max seconds to wait (0 = forever)

        Returns:
            List of broadcast envelopes when work arrives.
        """
        import time as _time
        start = _time.monotonic()
        since = self._state.get("last_broadcast_seen", 0.0) if self._state else 0.0

        while True:
            broadcasts = await self.check_broadcasts(since_ts=since)
            if broadcasts:
                # Advance watermark
                since = max(b.get("ts_score", 0.0) for b in broadcasts)
                if self._state:
                    self._state["last_broadcast_seen"] = since
                    self._state["pending_broadcasts"] = broadcasts
                return broadcasts

            if timeout > 0 and (_time.monotonic() - start) >= timeout:
                return []  # Timed out, no work

            await asyncio.sleep(poll_interval)

    async def check_broadcast_acks(self, broadcast_id: str) -> dict:
        """Check which agents have acknowledged a broadcast."""
        key = f"ada:broadcast:{self.project}:ack:{broadcast_id}"
        raw = await self.redis.cmd("HGETALL", key)
        if not raw:
            return {}
        # HGETALL returns [k1, v1, k2, v2, ...]
        acks = {}
        for i in range(0, len(raw), 2):
            acks[raw[i]] = json.loads(raw[i + 1])
        return acks

    # ── Convenience ────────────────────────────────────────────

    async def update_task(self, description: str, phase: str, progress: float):
        """Update current task in local state."""
        if self._state:
            self._state["current_task"] = {
                "description": description,
                "phase": phase,
                "progress": progress,
            }

    async def log_files(self, read: list[str] | None = None, written: list[str] | None = None):
        """Track files touched."""
        if self._state:
            ft = self._state.setdefault("files_touched", {"read": [], "written": []})
            if read:
                ft["read"] = list(set(ft["read"] + read))
            if written:
                ft["written"] = list(set(ft["written"] + written))

    async def get_session_log(self, n: int = 10) -> list[dict]:
        """Get last N session log entries."""
        results = await self.redis.cmd(
            "LRANGE", f"ada:bb:{self.project}:log", "0", str(n - 1)
        )
        return [json.loads(r) for r in (results or [])]

    async def get_session_snapshot(self, session_id: str) -> dict | None:
        """Retrieve a specific historical session."""
        return await self.redis.get_json(f"ada:session:{session_id}")

    # ── Internal ───────────────────────────────────────────────

    def _fresh_state(self) -> dict:
        return {
            "project": self.project,
            "session_id": self.session_id,
            "last_updated": _now(),
            "current_task": None,
            "consciousness": {
                "thinking_style": "analytical",
                "coherence": 0.5,
                "ice_cake_layers": 0,
            },
            "team": {"active": [self.agent_id], "parked": []},
            "decisions": [],
            "blockers": [],
            "files_touched": {"read": [], "written": []},
            "previous_sessions": [],
            "summary": "",
            "status": "FRESH",
            "ice_cake_layers": 0,
        }

    async def _check_inbox(self) -> list[dict]:
        """Non-destructive peek at inbox."""
        results = await self.redis.cmd(
            "LRANGE", f"ada:a2a:inbox:{self.agent_id}", "0", "-1"
        )
        return results or []

    async def close(self):
        await self.redis.close()
        await self.redis_hot.close()


# ── CLI Quick Test ─────────────────────────────────────────────

if __name__ == "__main__":
    import asyncio

    async def test():
        bb = Blackboard("test-project")

        # Hydrate
        state = await bb.hydrate()
        print(f"Hydrated: session={bb.session_id}, status={state['status']}")
        print(f"   Previous sessions: {len(state.get('previous_sessions', []))}")

        # Ice cake a decision
        await bb.ice_cake({
            "task": "Test decision",
            "rationale": "Testing blackboard",
            "gate": "FLOW",
            "gate_sd": 0.05,
        })
        count = await bb.get_ice_cake_count()
        print(f"Ice cake layers: {count}")

        # Update task
        await bb.update_task("Testing blackboard module", "test", 0.5)

        # Persist
        state["summary"] = "Tested blackboard module"
        await bb.persist(state)
        print(f"Persisted to Redis")

        # Read back
        log = await bb.get_session_log(3)
        print(f"Session log entries: {len(log)}")

        decisions = await bb.get_recent_decisions(3)
        print(f"Recent decisions: {len(decisions)}")

        await bb.close()
        print("\nBlackboard test complete!")

    asyncio.run(test())
