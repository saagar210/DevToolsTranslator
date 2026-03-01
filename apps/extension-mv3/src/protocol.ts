export type RedactionLevel = 'metadata_only' | 'redacted' | 'full';

export type CommandType =
  | 'cmd.list_tabs'
  | 'cmd.start_capture'
  | 'cmd.stop_capture'
  | 'cmd.set_ui_capture';

export type EventType =
  | 'evt.hello'
  | 'evt.tabs_list'
  | 'evt.session_started'
  | 'evt.raw_event'
  | 'evt.session_ended'
  | 'evt.error'
  | 'evt.pairing_discovered'
  | 'evt.pairing_approval_needed'
  | 'evt.pairing_established'
  | 'evt.pairing_revoked';

export type EnvelopeType = CommandType | EventType;

export type EventErrorCode =
  | 'already_attached'
  | 'permission_denied'
  | 'token_invalid'
  | 'ws_disconnected'
  | 'unsupported_command'
  | 'internal_error'
  | 'pairing_not_established';

export type PairingUxState =
  | 'not_paired'
  | 'discovering'
  | 'awaiting_approval'
  | 'paired'
  | 'reconnecting'
  | 'error';

export interface ControlEnvelopeV1<TPayload = unknown> {
  readonly v: 1;
  readonly type: EnvelopeType;
  readonly ts_ms: number;
  readonly token?: string;
  readonly request_id?: string;
  readonly correlation_id?: string;
  readonly session_id?: string;
  readonly event_seq?: number;
  readonly privacy_mode?: RedactionLevel;
  readonly payload: TPayload;
}

export interface RawEventPayload {
  readonly event_id: string;
  readonly cdp_method: string;
  readonly raw_event: Record<string, unknown>;
}

export interface TabDescriptor {
  readonly tab_id: number;
  readonly window_id: number;
  readonly url: string;
  readonly title: string;
  readonly active: boolean;
}

export interface SessionStartedPayload {
  readonly session_id: string;
  readonly tab_id: number;
  readonly privacy_mode: RedactionLevel;
  readonly started_at_ms: number;
}

export interface SessionEndedPayload {
  readonly session_id: string;
  readonly ended_at_ms: number;
}

export interface ErrorPayload {
  readonly code: EventErrorCode;
  readonly message: string;
  readonly details?: string;
  readonly session_id?: string;
}

export interface HelloPayload {
  readonly extension_version: string;
  readonly protocol_version: 1;
  readonly connected: boolean;
  readonly consent_enabled: boolean;
  readonly ui_capture_enabled: boolean;
  readonly active_session_id: string | null;
  readonly pairing_state?: PairingUxState;
  readonly trusted_device_id?: string | null;
}

export interface ListTabsPayload {
  readonly tabs: ReadonlyArray<TabDescriptor>;
}

export interface StartCaptureCommandPayload {
  readonly tab_id: number;
  readonly privacy_mode: RedactionLevel;
  readonly session_id: string;
  readonly enable_security_domain?: boolean;
}

export interface StopCaptureCommandPayload {
  readonly session_id: string;
}

export interface SetUiCaptureCommandPayload {
  readonly enabled: boolean;
}

export function nowMs(): number {
  return Date.now();
}

export function parseEnvelope(raw: string): ControlEnvelopeV1<unknown> {
  return JSON.parse(raw) as ControlEnvelopeV1<unknown>;
}

export function serializeEnvelope<TPayload>(envelope: ControlEnvelopeV1<TPayload>): string {
  return JSON.stringify(envelope);
}

export function makeHelloEvent(
  payload: HelloPayload,
  correlation_id?: string,
): ControlEnvelopeV1<HelloPayload> {
  return {
    v: 1,
    type: 'evt.hello',
    ts_ms: nowMs(),
    correlation_id,
    payload,
  };
}

export function makeTabsListEvent(
  payload: ListTabsPayload,
  correlation_id?: string,
): ControlEnvelopeV1<ListTabsPayload> {
  return {
    v: 1,
    type: 'evt.tabs_list',
    ts_ms: nowMs(),
    correlation_id,
    payload,
  };
}

export function makeSessionStartedEvent(
  payload: SessionStartedPayload,
  correlation_id?: string,
): ControlEnvelopeV1<SessionStartedPayload> {
  return {
    v: 1,
    type: 'evt.session_started',
    ts_ms: payload.started_at_ms,
    correlation_id,
    session_id: payload.session_id,
    privacy_mode: payload.privacy_mode,
    payload,
  };
}

export function makeSessionEndedEvent(
  payload: SessionEndedPayload,
  correlation_id?: string,
): ControlEnvelopeV1<SessionEndedPayload> {
  return {
    v: 1,
    type: 'evt.session_ended',
    ts_ms: payload.ended_at_ms,
    correlation_id,
    session_id: payload.session_id,
    payload,
  };
}

export function makeRawEventEnvelope(args: {
  readonly session_id: string;
  readonly event_seq: number;
  readonly privacy_mode: RedactionLevel;
  readonly payload: RawEventPayload;
}): ControlEnvelopeV1<RawEventPayload> {
  return {
    v: 1,
    type: 'evt.raw_event',
    ts_ms: nowMs(),
    session_id: args.session_id,
    event_seq: args.event_seq,
    privacy_mode: args.privacy_mode,
    payload: args.payload,
  };
}

export function makeErrorEvent(
  payload: ErrorPayload,
  correlation_id?: string,
): ControlEnvelopeV1<ErrorPayload> {
  return {
    v: 1,
    type: 'evt.error',
    ts_ms: nowMs(),
    correlation_id,
    session_id: payload.session_id,
    payload,
  };
}

export function mapAttachError(message: string | undefined): EventErrorCode {
  const lowered = (message ?? '').toLowerCase();
  if (lowered.includes('already attached')) {
    return 'already_attached';
  }
  if (lowered.includes('permission') || lowered.includes('not allowed')) {
    return 'permission_denied';
  }
  return 'internal_error';
}
