import type { ControlEnvelopeV1, RawEventPayload } from './protocol.js';

export const MAX_BUFFER_EVENTS = 5_000;
export const MAX_BUFFER_BYTES = 10 * 1024 * 1024;

export interface BufferDropReport {
  readonly dropped_events: number;
  readonly dropped_bytes: number;
}

interface QueueEntry {
  readonly envelope: ControlEnvelopeV1<RawEventPayload>;
  readonly byte_len: number;
}

export class CaptureBuffer {
  private readonly entries: QueueEntry[] = [];
  private totalBytes = 0;

  public push(envelope: ControlEnvelopeV1<RawEventPayload>): BufferDropReport {
    const byteLen = envelopeByteLength(envelope);
    this.entries.push({ envelope, byte_len: byteLen });
    this.totalBytes += byteLen;

    let droppedEvents = 0;
    let droppedBytes = 0;

    while (this.entries.length > MAX_BUFFER_EVENTS || this.totalBytes > MAX_BUFFER_BYTES) {
      const removed = this.entries.shift();
      if (!removed) {
        break;
      }
      droppedEvents += 1;
      droppedBytes += removed.byte_len;
      this.totalBytes -= removed.byte_len;
    }

    return { dropped_events: droppedEvents, dropped_bytes: droppedBytes };
  }

  public drain(): ControlEnvelopeV1<RawEventPayload>[] {
    const out = this.entries.map((entry) => entry.envelope);
    this.entries.length = 0;
    this.totalBytes = 0;
    return out;
  }

  public size(): number {
    return this.entries.length;
  }
}

export function envelopeByteLength(envelope: ControlEnvelopeV1<RawEventPayload>): number {
  return new TextEncoder().encode(JSON.stringify(envelope)).length;
}
