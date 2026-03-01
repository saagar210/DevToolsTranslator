#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const STAGE_ORDER = ['pct_5', 'pct_25', 'pct_50', 'pct_100'];

function parseArgs(argv) {
  const args = {
    dryRun: false,
    version: null,
    stage: 'pct_5',
    nowMs: Date.now(),
    telemetryStatus: 'pass',
    criticalAnomalies: 0,
    manualSmokeFile: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--dry-run') {
      args.dryRun = true;
    } else if (value === '--version' && argv[index + 1]) {
      args.version = argv[index + 1];
      index += 1;
    } else if (value === '--stage' && argv[index + 1]) {
      args.stage = argv[index + 1];
      index += 1;
    } else if (value === '--now-ms' && argv[index + 1]) {
      args.nowMs = Number(argv[index + 1]);
      index += 1;
    } else if (value === '--telemetry-status' && argv[index + 1]) {
      args.telemetryStatus = argv[index + 1];
      index += 1;
    } else if (value === '--critical-anomalies' && argv[index + 1]) {
      args.criticalAnomalies = Number(argv[index + 1]);
      index += 1;
    } else if (value === '--manual-smoke-file' && argv[index + 1]) {
      args.manualSmokeFile = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function hasManualSmokePass(filePath) {
  if (!fs.existsSync(filePath)) return false;
  const content = fs.readFileSync(filePath, 'utf8');
  return content
    .split('\n')
    .map((line) => line.trim().toLowerCase())
    .some(
      (line) =>
        line.startsWith('interactive_chrome_manual:') &&
        line.includes('pass') &&
        line.includes('date=20') &&
        line.includes('observer=') &&
        !line.includes('not_run'),
    );
}

function previousStage(stage) {
  const index = STAGE_ORDER.indexOf(stage);
  if (index <= 0) return null;
  return STAGE_ORDER[index - 1];
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const args = parseArgs(process.argv.slice(2));
  if (!args.version) {
    process.stderr.write('Missing required --version\n');
    process.exit(2);
  }
  if (!STAGE_ORDER.includes(args.stage)) {
    process.stderr.write(`Unsupported --stage ${args.stage}\n`);
    process.exit(2);
  }

  const policy = readJson(
    path.join(repoRoot, 'config', 'release', 'extension_rollout_policy.v2.json'),
  );
  const stageConfig = policy.stages.find((row) => row.stage === args.stage);
  if (!stageConfig) {
    process.stderr.write(`Stage policy missing for ${args.stage}\n`);
    process.exit(2);
  }

  const reasons = [];
  const manualSmokeFile =
    args.manualSmokeFile ?? path.join(repoRoot, 'docs', 'PHASE6_SMOKE_EVIDENCE.md');
  if (policy.blocking_rules.require_manual_smoke_pass && !hasManualSmokePass(manualSmokeFile)) {
    reasons.push('manual_smoke_missing');
  }

  const releaseManifestPath = path.join(
    repoRoot,
    'dist',
    'releases',
    'chrome-store-public',
    args.version,
    'extension',
    'release-manifest.v1.json',
  );
  if (!fs.existsSync(releaseManifestPath)) {
    reasons.push('compliance_failed');
  } else {
    const releaseManifest = readJson(releaseManifestPath);
    if (!releaseManifest?.compliance?.pass) {
      reasons.push('compliance_failed');
    }
  }

  if (args.telemetryStatus === 'fail') {
    reasons.push('telemetry_audit_failed');
  }
  if (args.criticalAnomalies > 0) {
    reasons.push('anomaly_budget_failed');
  }

  const updaterFeedPath = path.join(
    repoRoot,
    'dist',
    'releases',
    'update-feed',
    'staged_public_prerelease',
    'latest.json',
  );
  if (policy.blocking_rules.require_signature_verified) {
    if (!fs.existsSync(updaterFeedPath)) {
      reasons.push('signature_invalid');
    } else {
      const updaterFeed = readJson(updaterFeedPath);
      if (!updaterFeed.signature_verified) {
        reasons.push('signature_invalid');
      }
    }
  }

  const prevStage = previousStage(args.stage);
  if (prevStage) {
    const prevReportPath = path.join(
      repoRoot,
      'dist',
      'releases',
      'chrome-store-public',
      args.version,
      'publish',
      `publish-${prevStage}.v1.json`,
    );
    if (!fs.existsSync(prevReportPath)) {
      reasons.push('soak_incomplete');
    } else {
      const prevReport = readJson(prevReportPath);
      const elapsed = Number(args.nowMs) - Number(prevReport.published_at_ms ?? 0);
      const required = Number(stageConfig.min_soak_hours) * 60 * 60 * 1000;
      if (elapsed < required) {
        reasons.push('soak_incomplete');
      }
    }
  }

  const nonSoakReasons = reasons.filter((reason) => reason !== 'soak_incomplete');
  const action = nonSoakReasons.length > 0 ? 'block' : reasons.includes('soak_incomplete') ? 'pause' : 'advance';
  const status = nonSoakReasons.length > 0 ? 'fail' : reasons.includes('soak_incomplete') ? 'warn' : 'pass';

  const outputDir = path.join(
    repoRoot,
    'dist',
    'releases',
    'chrome-store-public',
    args.version,
    'controller',
  );
  fs.mkdirSync(outputDir, { recursive: true });
  const report = {
    v: 1,
    channel: 'chrome_store_public',
    version: args.version,
    stage: args.stage,
    dry_run: args.dryRun,
    action,
    status,
    reasons,
    evaluated_at_ms: Number(args.nowMs),
    policy_file: 'config/release/extension_rollout_policy.v2.json',
  };
  const reportPath = path.join(outputDir, `evaluate-${args.stage}.v1.json`);
  fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`, 'utf8');
  process.stdout.write(`${JSON.stringify({ status: 'ok', ...report, report_path: reportPath }, null, 2)}\n`);
}

main();
