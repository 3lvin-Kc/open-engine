import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
  },
  resolve: {
    alias: {
      '@openclaw/ts-client': path.resolve(__dirname, '../ts-client/src')
    }
  }
});
