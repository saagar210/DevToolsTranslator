export type EvidenceKindVm = 'raw_event' | 'net_row' | 'console' | 'derived_metric';
export type SessionSubviewVm = 'overview' | 'timeline' | 'network' | 'console' | 'findings' | 'export';

export interface EvidenceTargetVm {
  readonly session_id: string;
  readonly evidence_kind: EvidenceKindVm;
  readonly reference_id: string;
  readonly column?: string;
  readonly json_pointer?: string;
}

export interface EvidenceRouteVm {
  readonly path: string;
  readonly subview: SessionSubviewVm;
  readonly highlight_key: string;
}

export function resolveEvidenceRoute(target: EvidenceTargetVm): EvidenceRouteVm {
  const subview = subviewForEvidence(target.evidence_kind);
  const path = `/sessions/${target.session_id}/${subview}`;
  const highlight_key = [
    target.evidence_kind,
    target.reference_id,
    target.column ?? '',
    target.json_pointer ?? '',
  ].join(':');

  return { path, subview, highlight_key };
}

function subviewForEvidence(kind: EvidenceKindVm): SessionSubviewVm {
  switch (kind) {
    case 'raw_event':
      return 'timeline';
    case 'net_row':
      return 'network';
    case 'console':
      return 'console';
    case 'derived_metric':
      return 'findings';
    default:
      return 'overview';
  }
}
