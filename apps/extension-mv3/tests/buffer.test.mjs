import test from 'node:test';
import assert from 'node:assert/strict';
import { CaptureBuffer, MAX_BUFFER_EVENTS } from '../dist/buffer.js';

function makeEnvelope(seq) {
  return {
    v: 1,
    type: 'evt.raw_event',
    ts_ms: 1,
    session_id: 'sess_1',
    event_seq: seq,
    privacy_mode: 'metadata_only',
    payload: {
      event_id: `evt_${seq}`,
      cdp_method: 'Network.requestWillBeSent',
      raw_event: { method: 'Network.requestWillBeSent', params: { requestId: `${seq}` } },
    },
  };
}

test('capture buffer drops oldest items at event cap', () => {
  const buffer = new CaptureBuffer();
  let dropped = 0;

  for (let index = 1; index <= MAX_BUFFER_EVENTS + 2; index += 1) {
    dropped += buffer.push(makeEnvelope(index)).dropped_events;
  }

  assert.equal(dropped, 2);
  assert.equal(buffer.size(), MAX_BUFFER_EVENTS);
});
