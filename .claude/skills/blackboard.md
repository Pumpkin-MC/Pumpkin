# /blackboard — Redis Blackboard Operations

## Usage

```
/blackboard hydrate          # Start session — read state from Redis
/blackboard persist          # End session — write state to Redis
/blackboard ice-cake         # Ice-cake a decision
/blackboard status           # Show current blackboard state
/blackboard inbox            # Check agent inbox for handovers
/blackboard decisions [n]    # Show last N ice-caked decisions
/blackboard handover <agent> # Post handover to another agent
```

## Implementation

All operations use `blackboard.py` in the project root. The skill runs Python inline.

### hydrate

```bash
python3 -c "
import asyncio
from blackboard import Blackboard
async def main():
    bb = Blackboard('pumpkin', agent_id='$(cat .current-agent 2>/dev/null || echo orchestrator)')
    state = await bb.hydrate()
    print(f'Session: {bb.session_id}')
    print(f'Status: {state[\"status\"]}')
    print(f'Previous sessions: {len(state.get(\"previous_sessions\", []))}')
    pending = state.get('pending_handovers', [])
    if pending:
        print(f'INBOX: {len(pending)} pending handovers!')
    await bb.close()
asyncio.run(main())
"
```

### persist

```bash
python3 -c "
import asyncio
from blackboard import Blackboard
async def main():
    bb = Blackboard('pumpkin', agent_id='$(cat .current-agent 2>/dev/null || echo orchestrator)')
    state = await bb.hydrate()
    state['summary'] = input('Session summary: ') if not state.get('summary') else state['summary']
    await bb.persist(state)
    print('Persisted to Redis.')
    await bb.close()
asyncio.run(main())
"
```

### status

```bash
python3 -c "
import asyncio, json
from blackboard import Blackboard
async def main():
    bb = Blackboard('pumpkin')
    state = await bb.redis.get_json('ada:bb:pumpkin:state')
    if state:
        print(f'Last session: {state.get(\"session_id\")}')
        print(f'Last updated: {state.get(\"last_updated\")}')
        print(f'Status: {state.get(\"status\")}')
        print(f'Ice cake layers: {state.get(\"ice_cake_layers\", 0)}')
        print(f'Agent progress:')
        for k, v in state.get('agent_progress', {}).items():
            print(f'  {k}: {v*100:.0f}%')
    else:
        print('No blackboard state found. Run /blackboard hydrate first.')
    await bb.close()
asyncio.run(main())
"
```

### decisions

```bash
python3 -c "
import asyncio
from blackboard import Blackboard
async def main():
    bb = Blackboard('pumpkin')
    decisions = await bb.get_recent_decisions(${1:-10})
    for d in decisions:
        print(f'{d.get(\"id\", \"?\"): <12} {d.get(\"task\", \"?\")[:60]}')
    await bb.close()
asyncio.run(main())
"
```

### inbox

```bash
python3 -c "
import asyncio
from blackboard import Blackboard
async def main():
    agent = '$(cat .current-agent 2>/dev/null || echo orchestrator)'
    bb = Blackboard('pumpkin', agent_id=agent)
    handover = await bb.receive_handover()
    if handover:
        print(f'From: {handover[\"from_agent\"]}')
        print(f'Task: {handover[\"request\"][\"task\"]}')
        print(f'Expected: {handover[\"request\"][\"expected_output\"]}')
    else:
        print(f'No pending handovers for {agent}')
    await bb.close()
asyncio.run(main())
"
```

## Environment

Requires these env vars (no fallbacks):
- `UPSTASH_REDIS_REST_URL`
- `UPSTASH_REDIS_REST_TOKEN`
- `UPSTASH_REDIS_REST_URL2`
- `UPSTASH_REDIS_REST_TOKEN2`
