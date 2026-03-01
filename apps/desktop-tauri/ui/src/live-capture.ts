export type CaptureConnectionStatus = 'disconnected' | 'connecting' | 'connected';

export interface CaptureTabVm {
  readonly tab_id: number;
  readonly window_id: number;
  readonly url: string;
  readonly title: string;
  readonly active: boolean;
}

export interface BridgeDiagnosticVm {
  readonly ts_ms: number;
  readonly kind: string;
  readonly message: string;
}

export interface LiveCaptureSourceState {
  readonly connection_status: CaptureConnectionStatus;
  readonly consent_enabled: boolean;
  readonly ui_capture_enabled: boolean;
  readonly active_session_id: string | null;
  readonly tabs: readonly CaptureTabVm[];
  readonly diagnostics: readonly BridgeDiagnosticVm[];
}

export interface LiveCaptureViewModel {
  readonly connection_status: CaptureConnectionStatus;
  readonly consent_enabled: boolean;
  readonly ui_capture_enabled: boolean;
  readonly active_session_id: string | null;
  readonly tabs: readonly CaptureTabVm[];
  readonly diagnostics: readonly BridgeDiagnosticVm[];
  readonly can_start_capture: boolean;
  readonly can_stop_capture: boolean;
  readonly loading: boolean;
  readonly empty_reason: 'extension_unavailable' | 'no_tabs' | null;
  readonly error_message: string | null;
}

const ERROR_KINDS = new Set(['auth_reject', 'runtime_error', 'socket_error', 'error']);

export function buildLiveCaptureViewModel(source: LiveCaptureSourceState): LiveCaptureViewModel {
  const tabs = [...source.tabs].sort((left, right) => {
    if (left.tab_id !== right.tab_id) {
      return left.tab_id - right.tab_id;
    }
    return left.url.localeCompare(right.url);
  });

  const diagnostics = [...source.diagnostics].sort((left, right) => {
    if (left.ts_ms !== right.ts_ms) {
      return right.ts_ms - left.ts_ms;
    }
    if (left.kind !== right.kind) {
      return left.kind.localeCompare(right.kind);
    }
    return left.message.localeCompare(right.message);
  });

  const latestError = diagnostics.find((entry) => ERROR_KINDS.has(entry.kind));

  const hasActiveSession = source.active_session_id !== null;
  const canStart = source.connection_status === 'connected' && source.consent_enabled && !hasActiveSession;
  const canStop = hasActiveSession;
  const loading = source.connection_status === 'connecting';

  let emptyReason: LiveCaptureViewModel['empty_reason'] = null;
  if (source.connection_status !== 'connected') {
    emptyReason = 'extension_unavailable';
  } else if (tabs.length === 0) {
    emptyReason = 'no_tabs';
  }

  return {
    connection_status: source.connection_status,
    consent_enabled: source.consent_enabled,
    ui_capture_enabled: source.ui_capture_enabled,
    active_session_id: source.active_session_id,
    tabs,
    diagnostics,
    can_start_capture: canStart,
    can_stop_capture: canStop,
    loading,
    empty_reason: emptyReason,
    error_message: latestError?.message ?? null,
  };
}
