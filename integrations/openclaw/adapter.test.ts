import { describe, it, expect, vi, beforeEach, Mock } from 'vitest';

vi.stubGlobal('fetch', vi.fn());
const mockFetch = fetch as Mock;

describe('OpenClawAgent - OpenClaw API Integration', () => {
  let agent: any;

  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockClear();
  });

  it('should call OpenClaw Tools Invoke API correctly', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: { result: 'ok' } })
    });

    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://127.0.0.1:18789',
      openclawToken: 'test-token'
    });

    const result = await testAgent.invokeOpenClawTool({
      tool: 'web_search',
      args: { query: 'test' }
    });

    expect(mockFetch).toHaveBeenCalledWith(
      'http://127.0.0.1:18789/tools/invoke',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          'Authorization': 'Bearer test-token',
          'Content-Type': 'application/json'
        }),
        body: JSON.stringify({
          tool: 'web_search',
          action: 'json',
          args: { query: 'test' },
          sessionKey: 'main',
          dryRun: false
        })
      })
    );

    expect(result.success).toBe(true);
  });

  it('should return error when token is not configured', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawToken: ''
    });

    const result = await testAgent.invokeOpenClawTool({
      tool: 'test',
      args: {}
    });

    expect(result.success).toBe(false);
    expect(result.error).toContain('token not configured');
  });

  it('should handle HTTP errors from OpenClaw', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      statusText: 'Unauthorized'
    });

    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://127.0.0.1:18789',
      openclawToken: 'invalid-token'
    });

    const result = await testAgent.invokeOpenClawTool({
      tool: 'exec',
      args: { command: 'ls' }
    });

    expect(result.success).toBe(false);
    expect(result.error).toContain('HTTP error');
  });

  it('should handle network errors', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://127.0.0.1:18789',
      openclawToken: 'test-token'
    });

    const result = await testAgent.invokeOpenClawTool({
      tool: 'web_search',
      args: {}
    });

    expect(result.success).toBe(false);
    expect(result.error).toContain('Network error');
  });

  it('should pass custom sessionKey when provided', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ success: true })
    });

    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://127.0.0.1:18789',
      openclawToken: 'test-token'
    });

    await testAgent.invokeOpenClawTool({
      tool: 'sessions_show',
      args: {},
      sessionKey: 'my-session'
    });

    expect(mockFetch).toHaveBeenCalledWith(
      'http://127.0.0.1:18789/tools/invoke',
      expect.objectContaining({
        body: expect.stringContaining('"sessionKey":"my-session"')
      })
    );
  });

  it('should support dryRun option', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ success: true })
    });

    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://127.0.0.1:18789',
      openclawToken: 'test-token'
    });

    await testAgent.invokeOpenClawTool({
      tool: 'exec',
      args: { command: 'echo test' },
      dryRun: true
    });

    expect(mockFetch).toHaveBeenCalledWith(
      'http://127.0.0.1:18789/tools/invoke',
      expect.objectContaining({
        body: expect.stringContaining('"dryRun":true')
      })
    );
  });
});

describe('OpenClawAgent - Idempotency Key Generation', () => {
  it('should generate keys with tool name prefix', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030'
    });

    const key = (testAgent as any).generateKey('web_search', { query: 'test' });
    expect(key).toMatch(/^web_search-/);
  });

  it('should include params in key', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030'
    });

    const key = (testAgent as any).generateKey('exec', { command: 'ls' });
    expect(key).toContain('exec-');
  });
});

describe('OpenClawAgent - Configuration', () => {
  it('should use default OpenClaw URL when not provided', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030'
    });

    expect(testAgent.openclawUrl).toBe('http://127.0.0.1:18789');
  });

  it('should use custom OpenClaw URL when provided', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030',
      openclawUrl: 'http://custom:9999'
    });

    expect(testAgent.openclawUrl).toBe('http://custom:9999');
  });

  it('should initialize with empty sessionId', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030'
    });

    expect(testAgent.getSessionId()).toBe('');
  });

  it('should initialize with null goalId', async () => {
    const { OpenClawAgent } = await import('./adapter');
    const testAgent = new OpenClawAgent({
      userId: 'test',
      engineUrl: 'http://127.0.0.1:3030'
    });

    expect(testAgent.getGoalId()).toBeNull();
  });
});
