import type { RedactionLevel } from './protocol.js';

const SENSITIVE_HEADER_KEYS = new Set([
  'authorization',
  'cookie',
  'set-cookie',
  'proxy-authorization',
  'x-api-key',
  'api-key',
  'token',
]);

const BODY_FIELD_KEYS = new Set([
  'postdata',
  'post_data',
  'requestbody',
  'request_body',
  'responsebody',
  'response_body',
  'payloaddata',
  'payload_data',
  'body',
  'data',
  'text',
]);

export interface SanitizedEventResult {
  readonly raw_event: Record<string, unknown>;
  readonly body_limit_hit: boolean;
  readonly body_limit_len: number;
}

export function sanitizeHeaders(input: unknown): Record<string, string | string[]> {
  if (!input || typeof input !== 'object') {
    return {};
  }

  const entries = Object.entries(input as Record<string, unknown>);
  const output: Record<string, string | string[]> = {};
  for (const [key, value] of entries) {
    const lowered = key.toLowerCase();
    if (SENSITIVE_HEADER_KEYS.has(lowered)) {
      output[lowered] = '[redacted]';
      continue;
    }

    if (Array.isArray(value)) {
      output[lowered] = value.map((item) => String(item));
      continue;
    }

    output[lowered] = String(value);
  }

  return output;
}

export function sanitizeRawEvent(
  rawEvent: Record<string, unknown>,
  mode: RedactionLevel,
  maxBodyBytes = 2_000_000,
): SanitizedEventResult {
  const cloned = deepClone(rawEvent);
  const bodyState = { hit: false, maxLen: 0 };
  sanitizeInPlace(cloned, mode, bodyState, maxBodyBytes);
  return {
    raw_event: cloned,
    body_limit_hit: bodyState.hit,
    body_limit_len: bodyState.maxLen,
  };
}

function sanitizeInPlace(
  node: unknown,
  mode: RedactionLevel,
  bodyState: { hit: boolean; maxLen: number },
  maxBodyBytes: number,
): void {
  if (!node || typeof node !== 'object') {
    return;
  }

  if (Array.isArray(node)) {
    for (const item of node) {
      sanitizeInPlace(item, mode, bodyState, maxBodyBytes);
    }
    return;
  }

  const record = node as Record<string, unknown>;

  if (record.headers && typeof record.headers === 'object') {
    record.headers = sanitizeHeaders(record.headers);
  }

  for (const [key, value] of Object.entries(record)) {
    const lowered = key.toLowerCase();

    if (SENSITIVE_HEADER_KEYS.has(lowered)) {
      record[key] = '[redacted]';
      continue;
    }

    if (isBodyField(lowered)) {
      if (mode === 'metadata_only') {
        record[key] = '[omitted]';
        continue;
      }

      if (mode === 'redacted') {
        record[key] = '[redacted]';
        continue;
      }

      if (typeof value === 'string') {
        const bodyLen = new TextEncoder().encode(value).length;
        if (bodyLen > maxBodyBytes) {
          bodyState.hit = true;
          bodyState.maxLen = Math.max(bodyState.maxLen, bodyLen);
          record[key] = { len_bytes: bodyLen, stored: 'len_hash_only' };
          continue;
        }
      }
    }

    sanitizeInPlace(record[key], mode, bodyState, maxBodyBytes);
  }
}

function deepClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function isBodyField(loweredKey: string): boolean {
  return BODY_FIELD_KEYS.has(loweredKey);
}
