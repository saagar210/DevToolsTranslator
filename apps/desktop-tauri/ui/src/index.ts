export interface DesktopUiHealth {
  readonly status: 'ok';
  readonly subsystem: 'desktop-ui';
}

export const desktopUiHealth: DesktopUiHealth = {
  status: 'ok',
  subsystem: 'desktop-ui',
};

export { DesktopApp } from './App.js';
export type { DesktopAppProps } from './App.js';
export { AppLayout, sessionSubviews } from './router.js';
export { createDesktopClient, MockDesktopClient, TauriDesktopClient } from './api/client.js';
export type { DesktopClient } from './api/client.js';

export type {
  BridgeDiagnosticVm,
  CaptureConnectionStatus,
  CaptureTabVm,
  LiveCaptureSourceState,
  LiveCaptureViewModel,
} from './live-capture.js';
export { buildLiveCaptureViewModel } from './live-capture.js';

export type {
  EvidenceKindVm,
  EvidenceRouteVm,
  EvidenceTargetVm,
  SessionSubviewVm,
} from './evidence-routing.js';
export { resolveEvidenceRoute } from './evidence-routing.js';
