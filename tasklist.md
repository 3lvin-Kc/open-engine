# Task List: Automatic Tool Execution Tracking

## Task 1: Update OpenClaw Adapter
**Status**: PENDING
**Priority**: HIGH

### Subtasks:
- [ ] 1.1 Modify execute() method to not pass idempotency_key (let server auto-generate)
- [ ] 1.2 Remove or simplify the generateKey() method since server handles it
- [ ] 1.3 Test that auto-generated keys work correctly

## Task 2: Add Automatic Tool Tracking Helper
**Status**: PENDING
**Priority**: MEDIUM

### Subtasks:
- [ ] 2.1 Create a simple wrapTool() helper function
- [ ] 2.2 Add documentation on how to use it
- [ ] 2.3 Ensure it works with any tool function

## Task 3: Update Documentation
**Status**: PENDING
**Priority**: MEDIUM

### Subtasks:
- [ ] 3.1 Update FAILURE_CONTRACT.md to say "ALL tools protected"
- [ ] 3.2 Remove email/flight specific examples
- [ ] 3.3 Add "any tool" examples instead

## Task 4: Verify Implementation
**Status**: PENDING
**Priority**: HIGH

### Subtasks:
- [ ] 4.1 Run TypeScript build
- [ ] 4.2 Test adapter functionality
- [ ] 4.3 Verify idempotency still works

## Dependencies

Task 1 must complete before Task 4.
Task 3 can be done in parallel with Task 1-2.
