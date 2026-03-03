# OpenEngine: Automatic Tool Execution Tracking Plan

## Problem Statement

Currently, OpenClaw (the agent) needs to explicitly use idempotency keys to prevent duplicate tool executions. The user wants EVERY tool execution to be automatically tracked and protected from duplicates - without manual key management.

## Goal

Make OpenEngine automatically track ALL tool executions for OpenClaw, ensuring:
1. No duplicate tool runs (ever)
2. Server crashes don't lose progress
3. Agent resumes seamlessly after restart

## Implementation Strategy

### Phase 1: Simplify Adapter (Use Server Auto-Generated Keys)

The server now supports auto-generating idempotency keys. Update the OpenClaw adapter to:
- Remove manual key generation
- Let server handle key creation automatically
- Simplify the execute() API

### Phase 2: Add Automatic Tracking Utilities

Create helper utilities that make it trivial to wrap ANY tool execution:
- Simple track() function
- Auto-replay on restart
- Session state management

### Phase 3: Documentation

Update docs to reflect that ALL tools are protected, not just "important" ones.

## Files to Modify

1. integrations/openclaw/adapter.ts - Use server auto-generated keys
2. docs/FAILURE_CONTRACT.md - Update messaging
3. Create utility helpers if needed

## Success Criteria

- OpenClaw adapter uses server auto-generated idempotency keys
- Every tool call is automatically tracked
- Crash/restart recovery works out of the box
- Documentation reflects "all tools protected"
