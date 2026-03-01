import test from 'node:test';
import assert from 'node:assert/strict';
import { reconnectDelayMs } from '../dist/reconnect.js';

test('reconnectDelayMs follows deterministic schedule', () => {
  assert.equal(reconnectDelayMs(0), 1000);
  assert.equal(reconnectDelayMs(1), 2000);
  assert.equal(reconnectDelayMs(2), 5000);
  assert.equal(reconnectDelayMs(3), 10000);
  assert.equal(reconnectDelayMs(4), 10000);
  assert.equal(reconnectDelayMs(99), 10000);
});
