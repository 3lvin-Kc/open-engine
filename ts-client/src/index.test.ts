import { describe, it, expect } from 'vitest';

describe('ts-client', () => {
  it('should export StateEngineClient', async () => {
    const { StateEngineClient } = await import('../src/client');
    expect(StateEngineClient).toBeDefined();
  });
  
  it('should export types', async () => {
    const types = await import('../src/types');
    expect(types).toBeDefined();
  });
});
