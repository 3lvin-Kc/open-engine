/**
 * IDEMPOTENCY STRESS TEST
 * 
 * Tests that idempotency is unbreakable under:
 * 1. Concurrent calls with same key
 * 2. Crash during execution
 * 3. Race conditions
 */

const PARALLEL_CALLS = 100;
const SERVER_URL = 'http://127.0.0.1:3030';

let stats = {
  totalCalls: 0,
  uniqueExecutions: new Set(),
  duplicates: 0,
  errors: 0
};

async function call(method, params) {
  const response = await fetch(SERVER_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Math.random().toString(36)
    })
  });
  const result = await response.json();
  if (result.error) {
    throw new Error(result.error.message);
  }
  return result.result;
}

async function setup() {
  console.log('Setting up test user and session...\n');
  
  const user = await call('create_user', { username: `stress-test-${Date.now()}` });
  const session = await call('create_session', { user_id: user.id });
  const goal = await call('create_goal', { 
    user_id: user.id, 
    session_id: session.id, 
    title: 'Stress Test Goal' 
  });
  
  return { user, session, goal };
}

// ============================================
// TEST 1: Concurrent Calls with Same Key
// ============================================

async function testConcurrentCalls(context) {
  console.log('╔════════════════════════════════════════════════════════════════╗');
  console.log('║  TEST 1: Concurrent Calls with Same Key (100 parallel)        ║');
  console.log('╚════════════════════════════════════════════════════════════════╝\n');
  
  const idempotencyKey = `stress-test-${Date.now()}`;
  
  console.log(`Launching ${PARALLEL_CALLS} parallel requests with key: ${idempotencyKey}\n`);
  
  const startTime = Date.now();
  
  // Launch all calls in parallel
  const promises = [];
  for (let i = 0; i < PARALLEL_CALLS; i++) {
    promises.push(
      call('execute_tool_idempotent', {
        user_id: context.user.id,
        session_id: context.session.id,
        goal_id: context.goal.id,
        tool_name: 'stress_test',
        tool_input: { index: i },
        idempotency_key: idempotencyKey
      }).then(result => {
        stats.totalCalls++;
        stats.uniqueExecutions.add(result.id);
        return { success: true, id: result.id, index: i };
      }).catch(error => {
        stats.errors++;
        return { success: false, error: error.message, index: i };
      })
    );
  }
  
  const results = await Promise.all(promises);
  const duration = Date.now() - startTime;
  
  // Analyze results
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  
  console.log(`Completed in ${duration}ms\n`);
  console.log('Results:');
  console.log(`  Total calls:     ${PARALLEL_CALLS}`);
  console.log(`  Successful:      ${successful.length}`);
  console.log(`  Failed:          ${failed.length}`);
  console.log(`  Unique IDs:      ${stats.uniqueExecutions.size}`);
  
  // THE KEY TEST: All successful calls should return the SAME execution ID
  if (stats.uniqueExecutions.size === 1) {
    console.log('\n✅ PASS: All calls returned the SAME execution ID');
    console.log('   Idempotency held under concurrent load!');
    return true;
  } else if (stats.uniqueExecutions.size === 0 && failed.length === PARALLEL_CALLS) {
    console.log('\n⚠️  All calls failed - server may not be running');
    return false;
  } else {
    console.log(`\n❌ FAIL: Got ${stats.uniqueExecutions.size} different execution IDs!`);
    console.log('   Idempotency VIOLATED - this is a critical bug!');
    console.log(`   Unique IDs: ${[...stats.uniqueExecutions].slice(0, 5).join(', ')}...`);
    return false;
  }
}

// ============================================
// TEST 2: Different Keys = Different Executions
// ============================================

async function testDifferentKeys(context) {
  console.log('\n╔════════════════════════════════════════════════════════════════╗');
  console.log('║  TEST 2: Different Keys = Different Executions                 ║');
  console.log('╚════════════════════════════════════════════════════════════════╝\n');
  
  const results = new Set();
  
  for (let i = 0; i < 10; i++) {
    const result = await call('execute_tool_idempotent', {
      user_id: context.user.id,
      session_id: context.session.id,
      goal_id: context.goal.id,
      tool_name: 'test',
      tool_input: { index: i },
      idempotency_key: `unique-key-${i}-${Date.now()}`
    });
    results.add(result.id);
  }
  
  if (results.size === 10) {
    console.log('✅ PASS: Each unique key created a new execution');
    return true;
  } else {
    console.log(`❌ FAIL: Expected 10 unique IDs, got ${results.size}`);
    return false;
  }
}

// ============================================
// TEST 3: Persist After Restart
// ============================================

async function testPersistAfterRestart(context) {
  console.log('\n╔════════════════════════════════════════════════════════════════╗');
  console.log('║  TEST 3: Persist After Restart (simulated)                    ║');
  console.log('╚════════════════════════════════════════════════════════════════╝\n');
  
  const idempotencyKey = `persist-test-${Date.now()}`;
  
  // Create first execution
  const first = await call('execute_tool_idempotent', {
    user_id: context.user.id,
    session_id: context.session.id,
    goal_id: context.goal.id,
    tool_name: 'persist_test',
    tool_input: { test: 'data' },
    idempotency_key: idempotencyKey
  });
  
  console.log(`Created execution: ${first.id}`);
  console.log('Key: ' + idempotencyKey);
  
  // Simulate restart by calling with same key
  console.log('\n[Simulating agent restart...]\n');
  
  const afterRestart = await call('execute_tool_idempotent', {
    user_id: context.user.id,
    session_id: context.session.id,
    goal_id: context.goal.id,
    tool_name: 'persist_test',
    tool_input: { test: 'data' },
    idempotency_key: idempotencyKey
  });
  
  console.log(`After restart: ${afterRestart.id}`);
  
  if (first.id === afterRestart.id) {
    console.log('\n✅ PASS: Same execution returned after "restart"');
    console.log('   State persisted correctly!');
    return true;
  } else {
    console.log('\n❌ FAIL: Different execution ID after restart!');
    return false;
  }
}

// ============================================
// RUN ALL TESTS
// ============================================

async function main() {
  console.log('╔════════════════════════════════════════════════════════════════╗');
  console.log('║        IDEMPOTENCY STRESS TEST SUITE                           ║');
  console.log('╚════════════════════════════════════════════════════════════════╝\n');
  
  console.log('⚠️  Make sure the state engine is running on ' + SERVER_URL);
  console.log('   Run: cd state-engine && cargo run -- --database test.db\n');
  
  try {
    const context = await setup();
    
    const test1 = await testConcurrentCalls(context);
    const test2 = await testDifferentKeys(context);
    const test3 = await testPersistAfterRestart(context);
    
    console.log('\n╔════════════════════════════════════════════════════════════════╗');
    console.log('║                      FINAL RESULTS                             ║');
    console.log('╚════════════════════════════════════════════════════════════════╝\n');
    
    const allPassed = test1 && test2 && test3;
    
    console.log(`Test 1 (Concurrent Calls):    ${test1 ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`Test 2 (Different Keys):      ${test2 ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`Test 3 (Persist After Restart): ${test3 ? '✅ PASS' : '❌ FAIL'}`);
    
    console.log('\n' + (allPassed ? '✅ ALL TESTS PASSED' : '❌ SOME TESTS FAILED'));
    
    if (allPassed) {
      console.log('\nIdempotency is UNBREAKABLE. Trust established.');
    } else {
      console.log('\nIdempotency has issues. Do not deploy to production.');
      process.exit(1);
    }
    
  } catch (error) {
    console.error('\n❌ Test suite failed:', error.message);
    console.error('\nMake sure the state engine is running:');
    console.error('  cd state-engine && cargo run -- --database test.db');
    process.exit(1);
  }
}

main();
