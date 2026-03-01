import test from 'node:test';
import assert from 'node:assert/strict';
import { sanitizeHeaders, sanitizeRawEvent } from '../dist/privacy.js';

test('sanitizeHeaders lowercases keys and redacts sensitive values', () => {
  const headers = sanitizeHeaders({ Authorization: 'Bearer abc', Accept: 'application/json' });
  assert.equal(headers.authorization, '[redacted]');
  assert.equal(headers.accept, 'application/json');
});

test('metadata_only strips body fields', () => {
  const result = sanitizeRawEvent(
    {
      method: 'Network.requestWillBeSent',
      params: {
        request: {
          headers: { Cookie: 'a=b', Accept: 'application/json' },
          postData: 'secret',
        },
      },
    },
    'metadata_only',
  );

  assert.equal(result.raw_event.params.request.headers.cookie, '[redacted]');
  assert.equal(result.raw_event.params.request.postData, '[omitted]');
});
