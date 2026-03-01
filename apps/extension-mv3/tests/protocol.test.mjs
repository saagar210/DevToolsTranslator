import test from 'node:test';
import assert from 'node:assert/strict';
import { mapAttachError } from '../dist/protocol.js';

test('mapAttachError maps already attached errors', () => {
  assert.equal(mapAttachError('Another debugger is already attached'), 'already_attached');
});

test('mapAttachError maps permission errors', () => {
  assert.equal(mapAttachError('Permission denied for debugger attach'), 'permission_denied');
});
