# Why This Exists

## The Problem

AI agents fail in production. A lot.

Not because they're stupid. Because they have no memory.

### What Goes Wrong

**The Duplicate Email**
Agent tries to send an email. Network times out. Agent retries. User gets two emails. Or three. Or ten.

**The Lost Task**
Agent is working on "book a flight to NYC." Server restarts. Agent forgets what it was doing. User has to start over.

**The Mystery Action**
User asks "Why did you delete that file?" Agent has no idea. No logs, no history, no explanation.

**The Wasted API Call**
Agent calls an expensive API three times because it wasn't sure if the first call worked. That's real money.

**The Half-Done Job**
Agent creates a calendar event, crashes before confirming, leaves the user's schedule in a weird state.

## The Root Cause

Agents keep state in memory. Memory is:

- Lost on restart
- Lost on crash
- Lost on timeout
- Lost on deployment
- Lost on scale-up

Every restart is a clean slate. Every crash is amnesia.

## What This Engine Does

It gives agents a place to remember things that survives crashes.

### The Contract

If you use this engine, these failures will not happen:

1. **Tool executions are idempotent** - Same action, same key, same result. No duplicates.
2. **Goals survive restart** - What the agent was doing is still there when it wakes up.
3. **Duplicate actions are detectable** - Check before you act.
4. **State transitions are auditable** - Every step has a timestamp and a chain of custody.
5. **Partial failures don't corrupt state** - All-or-nothing writes. No half-states.

## What This Engine Is Not

It's not a vector database. It's not a message queue. It's not a workflow engine.

It's one thing: **durable state for agents that can't afford to forget.**

## The Minimal Example

```typescript
// Before: Agent in memory
let tasks = [];  // Lost on restart

// After: Agent with engine
const goal = await client.createGoal(userId, sessionId, "Book flight");
const exec = await client.executeToolIdempotent(
  userId, sessionId, goal.id,
  "search_flights",
  { destination: "NYC" },
  "search-nyc-flights-2024-03-15"  // This key prevents duplicates
);

// If agent crashes and restarts:
const goals = await client.listPendingGoals(sessionId);
// Goals are still there. Resume where you left off.
```

## When You Need This

- Your agent makes real-world changes (emails, payments, bookings)
- Your agent runs tasks that take longer than a request
- Your agent needs to explain what it did and why
- Your agent costs money per API call
- You're tired of debugging "I don't know what happened"

## When You Don't Need This

- Your agent is stateless and idempotent by design
- Your agent only reads data
- You don't mind if tasks get repeated
- You're okay with "I don't know" as an answer

---

**Bottom line:** If your agent does things that cost money or can't be undone, you need this.
