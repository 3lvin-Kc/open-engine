# Failure Demo

This demonstrates real agent failures and how the state engine prevents them.

## Running the Demo

```bash
cd examples/failure-demo
node demo.js
```

## What It Shows

### Scenario 1: Network Timeout + Retry

**Without Engine:**
```
Attempt 1: Send email to user@example.com
[Agent thinks: "Did that work? Network timed out..."]
Attempt 2: Send email to user@example.com (retry)

RESULT: User receives 2 identical emails
```

**With Engine:**
```
Attempt 1: Send email → Execution recorded with idempotency key
Attempt 2: Same idempotency key → Duplicate prevented!

RESULT: User receives exactly 1 email
```

### Scenario 2: Crash Mid-Execution

**Without Engine:**
```
[Agent starts: Send 3 welcome emails]
Email 1: Sent ✓
Email 2: Sent ✓
💥 CRASH
[Agent restarts with no memory]
[Agent starts over]
Email 1: Sent again ❌ (duplicate)
Email 2: Sent again ❌ (duplicate)
Email 3: Sent ✓

RESULT: Users 1 and 2 get duplicate emails
```

**With Engine:**
```
[Agent starts: Send 3 welcome emails]
Email 1: Sent ✓ (recorded)
Email 2: Sent ✓ (recorded)
💥 CRASH
[Agent restarts, queries engine]
Engine: "Emails 1 and 2 already done"
[Agent resumes from email 3]
Email 1: Duplicate prevented ✓
Email 2: Duplicate prevented ✓
Email 3: Sent ✓

RESULT: Each user gets exactly 1 email
```

## Key Insight

The difference is **state that survives failures**.

| Event | Without Engine | With Engine |
|-------|----------------|-------------|
| Network timeout | Retry = duplicate | Retry = same result |
| Crash | Memory lost | State persisted |
| Restart | Start over | Resume |
| Audit | "I don't know" | Full history |

## Real-World Impact

This demo simulates what happens in production:

- **Email agents** sending duplicate messages
- **Payment agents** charging twice
- **Booking agents** making double reservations
- **Notification agents** spamming users

All prevented by one thing: **idempotent state management**.
