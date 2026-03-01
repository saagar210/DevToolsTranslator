#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function parseArgs(argv) {
  const args = {
    dryRun: false,
    version: null,
    stage: 'pct_5',
    approvalFile: null,
    evidencePackPath: null,
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
    } else if (value === '--approval-file' && argv[index + 1]) {
      args.approvalFile = argv[index + 1];
      index += 1;
    } else if (value === '--evidence-pack-path' && argv[index + 1]) {
      args.evidencePackPath = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function requireEnv(name) {
  const value = process.env[name];
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : null;
}

function stagePct(stage) {
  switch (stage) {
    case 'pct_5':
      return 5;
    case 'pct_25':
      return 25;
    case 'pct_50':
      return 50;
    case 'pct_100':
      return 100;
    default:
      throw new Error(`unsupported stage: ${stage}`);
  }
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const { dryRun, version, stage, approvalFile, evidencePackPath } = parseArgs(
    process.argv.slice(2),
  );
  if (!version) {
    process.stderr.write('Missing required --version\n');
    process.exit(2);
  }

  const packagedManifest = path.join(
    repoRoot,
    'dist',
    'releases',
    'chrome-store-public',
    version,
    'extension',
    'release-manifest.v1.json',
  );
  if (!fs.existsSync(packagedManifest)) {
    process.stdout.write(
      `${JSON.stringify(
        {
          status: 'error',
          error_code: 'extension_package_missing',
          message: `Missing ${packagedManifest}`,
        },
        null,
        2,
      )}\n`,
    );
    process.exit(1);
  }

  const requiredCreds = [
    'CWS_CLIENT_ID',
    'CWS_CLIENT_SECRET',
    'CWS_REFRESH_TOKEN',
    'CWS_EXTENSION_ID',
  ];
  const missingCreds = requiredCreds.filter((name) => !requireEnv(name));
  if (!dryRun && missingCreds.length > 0) {
    process.stdout.write(
      `${JSON.stringify(
        {
          status: 'error',
          error_code: 'cws_credentials_missing',
          missing: missingCreds,
          message: 'Chrome Web Store credentials are required for non-dry-run publish.',
        },
        null,
        2,
      )}\n`,
    );
    process.exit(1);
  }

  if (!dryRun) {
    if (!approvalFile || !fs.existsSync(approvalFile)) {
      process.stdout.write(
        `${JSON.stringify(
          {
            status: 'error',
            error_code: 'missing_stage_approval',
            message: 'A rollout controller approval file is required for non-dry-run publish.',
          },
          null,
          2,
        )}\n`,
      );
      process.exit(1);
    }
    const approval = JSON.parse(fs.readFileSync(approvalFile, 'utf8'));
    if (!approval.approved) {
      process.stdout.write(
        `${JSON.stringify(
          {
            status: 'error',
            error_code: 'rollout_blocked',
            message: 'Rollout controller denied stage advance.',
            reasons: approval.reasons ?? [],
          },
          null,
          2,
        )}\n`,
      );
      process.exit(1);
    }
  }

  const outputDir = path.join(
    repoRoot,
    'dist',
    'releases',
    'chrome-store-public',
    version,
    'publish',
  );
  fs.mkdirSync(outputDir, { recursive: true });

  const report = {
    v: 1,
    status: 'ok',
    dry_run: dryRun,
    channel: 'chrome_store_public',
    version,
    stage,
    rollout_pct: stagePct(stage),
    cws_item_id: requireEnv('CWS_EXTENSION_ID') ?? 'dry_run_extension',
    compliance_manifest: packagedManifest,
    approval_file: approvalFile,
    evidence_pack_path: evidencePackPath,
    published_at_ms: Date.now(),
  };

  const reportPath = path.join(outputDir, `publish-${stage}.v1.json`);
  fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`, 'utf8');
  process.stdout.write(`${JSON.stringify({ ...report, report_path: reportPath }, null, 2)}\n`);
}

main();
