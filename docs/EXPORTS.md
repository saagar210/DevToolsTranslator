# Export Bundle Specification v1.0

## Bundle Output
- Format: zip archive.
- Deterministic layout and file ordering.
- Integrity based on BLAKE3 hashes of stored file bytes.

## Root Layout (Locked)
```text
manifest.json
session.json
normalized/*.ndjson + indexes
analysis/*.ndjson
raw/events.ndjson.zst + index
blobs/ (full export only)
report/report.html + report.json
integrity/files.blake3.json + bundle.blake3.txt
```

## manifest.json Schema
```json
{
  "v": 1,
  "session_id": "string",
  "exported_at_ms": 0,
  "privacy_mode": "metadata_only|redacted|full",
  "export_profile": "share_safe|full",
  "files": [
    {
      "path": "normalized/network_requests.ndjson",
      "kind": "normalized|analysis|raw|blob|report|integrity|index",
      "line_count": 0,
      "sha_blake3": "hex"
    }
  ],
  "indexes": [
    {
      "name": "raw/events.index.ndjson",
      "maps_file": "raw/events.ndjson.zst",
      "mode": "line|line+byte"
    }
  ],
  "evidence_indexes": {
    "raw_event": "raw/events.index.ndjson",
    "net_row": "normalized/network.index.ndjson",
    "console": "normalized/console.index.ndjson",
    "derived_metric": "analysis/derived_metrics.index.ndjson"
  }
}
```

## NDJSON Formats
### normalized/*.ndjson
- One canonical JSON object per line.
- Deterministic sort order by natural key + deterministic ID.
- Includes network, console, lifecycle, interactions, and membership records.

### analysis/*.ndjson
- findings, claims, evidence_refs, and derived metrics.
- Must retain claim/evidence ordering used in app.

### raw/events.ndjson.zst
- Compressed raw events stream.
- Event lines correspond to `events_raw` canonical record shape.

## Index Formats
### Line index (required)
- Maps deterministic record key to line number.
- Used for evidence pointer resolution.

### Byte offset index (optional)
- For compressed or large files may include `byte_offset_start/end`.
- Must not alter line-based lookup semantics.

## Evidence Resolution In Exports
1. Resolve evidence kind from `EvidenceRef.kind`.
2. Use `manifest.evidence_indexes[kind]`.
3. Lookup target record by deterministic key.
4. Apply `json_pointer` for field-level highlighting.
5. For absence evidence, verify `container_hash` against indexed container hash.

## Integrity Chain
### Per-file integrity
- `integrity/files.blake3.json` contains hash per file path over stored bytes.

### Bundle integrity
- `integrity/bundle.blake3.txt` contains BLAKE3 over canonical concatenation string:
- each line `hash  path` sorted by path.

## Share-Safe vs Full Rules
### Share-safe default
- Excludes `blobs/` completely.
- Excludes or redacts secret-bearing headers and payload segments.
- Evidence previews must be redacted or omitted.

### Full export
- Includes blobs when privacy mode permits.
- Blocked when session privacy mode is `metadata_only`.
- Requires explicit user confirmation in UI.

## Blob Rules
- Blob files are content-addressed by BLAKE3.
- Blob manifest includes byte length, media type, and source linkage.
- Share-safe exports must not include raw blob payload bytes.

## Failure Handling
- If integrity file generation fails, export is marked failed and output is not surfaced as complete.
- If any required index is missing, export is marked invalid and cannot be opened in-app.
