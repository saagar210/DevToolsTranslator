#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

function parseArgs(argv) {
  const args = { input: null };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--input' && argv[index + 1]) {
      args.input = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function classifyDrift(driftPct, warnPct, failPct) {
  if (driftPct > failPct) {
    return 'fail';
  }
  if (driftPct > warnPct) {
    return 'warn';
  }
  return 'pass';
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const thresholds = JSON.parse(
    fs.readFileSync(path.join(repoRoot, 'config', 'perf.thresholds.v1.json'), 'utf8'),
  );
  const { input } = parseArgs(process.argv.slice(2));

  const measured = input
    ? JSON.parse(fs.readFileSync(path.resolve(input), 'utf8'))
    : {
        sustained_capture_memory_peak_bytes: 805306368,
        bundle_inspect_resolve_p95_ms: 245,
        normalization_events_per_s: 2550,
      };

  const warnPct = thresholds.policy.warn_drift_pct;
  const failPct = thresholds.policy.fail_drift_pct;

  const checks = [];

  const baselineMemory = thresholds.baselines.sustained_capture_memory_peak_bytes;
  const memory = measured.sustained_capture_memory_peak_bytes;
  const memoryDrift = ((memory - baselineMemory) / baselineMemory) * 100;
  checks.push({
    key: 'sustained_capture_memory_peak_bytes',
    measured: memory,
    baseline: baselineMemory,
    drift_pct: Number(memoryDrift.toFixed(2)),
    cap: thresholds.hard_caps.sustained_capture_memory_peak_bytes,
    status:
      memory > thresholds.hard_caps.sustained_capture_memory_peak_bytes
        ? 'fail'
        : classifyDrift(memoryDrift, warnPct, failPct),
  });

  const baselineP95 = thresholds.baselines.bundle_inspect_resolve_p95_ms;
  const p95 = measured.bundle_inspect_resolve_p95_ms;
  const p95Drift = ((p95 - baselineP95) / baselineP95) * 100;
  checks.push({
    key: 'bundle_inspect_resolve_p95_ms',
    measured: p95,
    baseline: baselineP95,
    drift_pct: Number(p95Drift.toFixed(2)),
    cap: thresholds.hard_caps.bundle_inspect_resolve_p95_ms,
    status:
      p95 > thresholds.hard_caps.bundle_inspect_resolve_p95_ms
        ? 'fail'
        : classifyDrift(p95Drift, warnPct, failPct),
  });

  const baselineThroughput = thresholds.baselines.normalization_events_per_s;
  const throughput = measured.normalization_events_per_s;
  const throughputDrift = ((baselineThroughput - throughput) / baselineThroughput) * 100;
  checks.push({
    key: 'normalization_events_per_s',
    measured: throughput,
    baseline: baselineThroughput,
    drift_pct: Number(throughputDrift.toFixed(2)),
    floor: thresholds.hard_caps.normalization_min_events_per_s,
    status:
      throughput < thresholds.hard_caps.normalization_min_events_per_s
        ? 'fail'
        : classifyDrift(throughputDrift, warnPct, failPct),
  });

  const hasFail = checks.some((check) => check.status === 'fail');
  const hasWarn = checks.some((check) => check.status === 'warn');

  const result = {
    v: 1,
    policy: thresholds.policy,
    overall: hasFail ? 'fail' : hasWarn ? 'warn' : 'pass',
    checks,
  };

  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  process.exit(hasFail ? 1 : 0);
}

main();
