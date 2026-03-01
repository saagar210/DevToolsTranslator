interface UiState {
  readonly pairing: {
    port: number | null;
    token: string | null;
    trusted_device_id: string | null;
    trusted: boolean;
    pairing_state:
      | 'not_paired'
      | 'discovering'
      | 'awaiting_approval'
      | 'paired'
      | 'reconnecting'
      | 'error';
  };
  readonly pairing_state:
    | 'not_paired'
    | 'discovering'
    | 'awaiting_approval'
    | 'paired'
    | 'reconnecting'
    | 'error';
  readonly trusted: boolean;
  readonly trusted_device_id: string | null;
  readonly connection_status: 'disconnected' | 'connecting' | 'connected';
  readonly consent_enabled: boolean;
  readonly ui_capture_enabled: boolean;
  readonly active_session_id: string | null;
  readonly buffered_events: number;
}

const statusLabel = document.querySelector<HTMLElement>('#status-label');
const helperLabel = document.querySelector<HTMLElement>('#status-helper');
const activeSessionLabel = document.querySelector<HTMLElement>('#active-session');
const bufferedEventsLabel = document.querySelector<HTMLElement>('#buffered-events');
const consentInput = document.querySelector<HTMLInputElement>('#consent-enabled');
const uiCaptureInput = document.querySelector<HTMLInputElement>('#ui-capture-enabled');
const discoverButton = document.querySelector<HTMLButtonElement>('#find-desktop-app');
const connectButton = document.querySelector<HTMLButtonElement>('#connect');
const disconnectButton = document.querySelector<HTMLButtonElement>('#disconnect');
const openDesktopButton = document.querySelector<HTMLButtonElement>('#open-desktop');
const openSidePanelButton = document.querySelector<HTMLButtonElement>('#open-side-panel');
const refreshButton = document.querySelector<HTMLButtonElement>('#refresh-state');
const portInput = document.querySelector<HTMLInputElement>('#pairing-port');
const tokenInput = document.querySelector<HTMLInputElement>('#pairing-token');
const savePairingButton = document.querySelector<HTMLButtonElement>('#save-pairing');
const trustedDeviceLabel = document.querySelector<HTMLElement>('#trusted-device');

function setStatus(message: string, helper = ''): void {
  if (statusLabel) {
    statusLabel.textContent = message;
  }
  if (helperLabel) {
    helperLabel.textContent = helper;
  }
}

async function sendMessage<T>(message: Record<string, unknown>): Promise<T> {
  const response = await chrome.runtime.sendMessage(message);
  return response as T;
}

function statusCopy(state: UiState): { label: string; helper: string } {
  if (state.connection_status === 'connected') {
    if (state.active_session_id) {
      return {
        label: 'Connected and capturing',
        helper: 'Capture is active. Open the desktop app to stop and review.',
      };
    }
    return {
      label: 'Connected',
      helper: 'You are ready. Choose a tab in the desktop app and start capture.',
    };
  }
  if (state.connection_status === 'connecting' || state.pairing_state === 'reconnecting') {
    return {
      label: 'Reconnecting',
      helper: 'Waiting for Desktop App. Keep the app open and retry if needed.',
    };
  }
  if (state.pairing_state === 'error') {
    return {
      label: 'Connection needs attention',
      helper: 'Try Find Desktop App, then Connect. Use Advanced if needed.',
    };
  }
  if (!state.pairing.token) {
    return {
      label: 'Not paired yet',
      helper: 'Click Find Desktop App or use Advanced setup one time.',
    };
  }
  return {
    label: 'Disconnected',
    helper: 'Pairing is saved. Click Connect to rejoin the desktop app.',
  };
}

async function refreshState(): Promise<UiState | null> {
  type Response = { ok: boolean; error?: string; state?: UiState };
  const response = await sendMessage<Response>({ action: 'state.get' });
  if (!response.ok || !response.state) {
    setStatus('Unable to load extension state', response.error ?? 'Unknown error');
    return null;
  }

  const next = response.state;
  if (consentInput) {
    consentInput.checked = next.consent_enabled;
  }
  if (uiCaptureInput) {
    uiCaptureInput.checked = next.ui_capture_enabled;
  }
  if (activeSessionLabel) {
    activeSessionLabel.textContent = next.active_session_id ?? 'none';
  }
  if (bufferedEventsLabel) {
    bufferedEventsLabel.textContent = String(next.buffered_events);
  }
  if (trustedDeviceLabel) {
    trustedDeviceLabel.textContent = next.trusted_device_id ?? 'none';
  }
  if (portInput) {
    portInput.value = next.pairing.port ? String(next.pairing.port) : '';
  }
  if (tokenInput) {
    tokenInput.value = next.pairing.token ?? '';
  }

  const copy = statusCopy(next);
  setStatus(copy.label, copy.helper);
  return next;
}

async function discoverDesktopApp(): Promise<void> {
  type Response = { ok: boolean; error?: string; state?: UiState };
  const response = await sendMessage<Response>({ action: 'pairing.discover' });
  if (!response.ok) {
    setStatus('Could not find Desktop App', response.error ?? 'Start Desktop App and retry.');
  }
  await refreshState();
}

async function savePairingFallback(): Promise<void> {
  if (!portInput || !tokenInput) {
    return;
  }
  const port = Number(portInput.value);
  const token = tokenInput.value.trim();
  type Response = { ok: boolean; error?: string };
  const response = await sendMessage<Response>({ action: 'pairing.set', port, token });
  if (!response.ok) {
    setStatus('Could not save advanced pairing', response.error ?? 'Invalid pairing values.');
    return;
  }
  setStatus('Advanced pairing saved', 'You can now click Connect.');
  await refreshState();
}

async function connect(): Promise<void> {
  await sendMessage({ action: 'ws.connect' });
  await refreshState();
}

async function disconnect(): Promise<void> {
  await sendMessage({ action: 'ws.disconnect' });
  await refreshState();
}

async function setConsent(enabled: boolean): Promise<void> {
  await sendMessage({ action: 'consent.set', enabled });
  await refreshState();
}

async function setUiCapture(enabled: boolean): Promise<void> {
  await sendMessage({ action: 'ui_capture.set', enabled });
  await refreshState();
}

async function openDesktopApp(): Promise<void> {
  type Response = { ok: boolean; error?: string };
  const response = await sendMessage<Response>({ action: 'desktop.open' });
  if (!response.ok) {
    setStatus('Desktop launch not detected', response.error ?? 'Open desktop app manually.');
  } else {
    setStatus('Launch signal sent', 'If app did not open, start it from Applications.');
  }
}

async function openSidePanel(): Promise<void> {
  await sendMessage({ action: 'sidepanel.open' });
}

discoverButton?.addEventListener('click', () => {
  void discoverDesktopApp();
});
connectButton?.addEventListener('click', () => {
  void connect();
});
disconnectButton?.addEventListener('click', () => {
  void disconnect();
});
openDesktopButton?.addEventListener('click', () => {
  void openDesktopApp();
});
openSidePanelButton?.addEventListener('click', () => {
  void openSidePanel();
});
refreshButton?.addEventListener('click', () => {
  void refreshState();
});
savePairingButton?.addEventListener('click', () => {
  void savePairingFallback();
});
consentInput?.addEventListener('change', () => {
  void setConsent(Boolean(consentInput.checked));
});
uiCaptureInput?.addEventListener('change', () => {
  void setUiCapture(Boolean(uiCaptureInput.checked));
});

void refreshState();
