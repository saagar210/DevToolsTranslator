#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function parseArgs(argv) {
  const args = {
    dryRun: false,
    version: null,
    channel: 'public_stable',
    fromStage: null,
    toStage: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--dry-run') {
      args.dryRun = true;
    } else if (value === '--version' && argv[index + 1]) {
      args.version = argv[index + 1];
      index += 1;
    } else if (value === '--channel' && argv[index + 1]) {
      args.channel = argv[index + 1];
      index += 1;
    } else if (value === '--from-stage' && argv[index + 1]) {
      args.fromStage = argv[index + 1];
      index += 1;
    } else if (value === '--to-stage' && argv[index + 1]) {
      args.toStage = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const args = parseArgs(process.argv.slice(2));
  if (!args.version || !args.fromStage || !args.toStage) {
    process.stderr.write('Missing required --version --from-stage --to-stage\n');
    process.exit(2);
  }

  const evalPath = path.join(
    repoRoot,
    'dist',
    'releases',
    'update-feed',
    args.channel,
    'controller',
    `evaluate-${args.fromStage}.v1.json`,
  );
  if (!fs.existsSync(evalPath)) {
    process.stdout.write(
      `${JSON.stringify(
        {
          status: 'error',
          error_code: 'missing_updater_stage_evaluation',
          message: `Missing ${evalPath}`,
        },
        null,
        2,
      )}\n`,
    );
    process.exit(1);
  }

  const evaluation = readJson(evalPath);
  if (!args.dryRun && evaluation.action !== 'advance') {
    process.stdout.write(
      `${JSON.stringify(
        {
          status: 'error',
          error_code: 'updater_rollout_blocked',
          action: evaluation.action,
          reasons: evaluation.reasons ?? [],
        },
        null,
        2,
      )}\n`,
    );
    process.exit(1);
  }

  const outDir = path.join(
    repoRoot,
    'dist',
    'releases',
    'update-feed',
    args.channel,
    'controller',
  );
  fs.mkdirSync(outDir, { recursive: true });
  const approval = {
    v: 1,
    channel: args.channel,
    version: args.version,
    from_stage: args.fromStage,
    to_stage: args.toStage,
    dry_run: args.dryRun,
    approved: evaluation.action === 'advance',
    controller_action: evaluation.action,
    reasons: evaluation.reasons ?? [],
    approved_at_ms: Date.now(),
  };
  const approvalPath = path.join(outDir, `advance-${args.toStage}.v1.json`);
  fs.writeFileSync(approvalPath, `${JSON.stringify(approval, null, 2)}\n`, 'utf8');

  process.stdout.write(
    `${JSON.stringify(
      {
        status: 'ok',
        dry_run: args.dryRun,
        channel: args.channel,
        version: args.version,
        from_stage: args.fromStage,
        to_stage: args.toStage,
        approval_path: approvalPath,
      },
      null,
      2,
    )}\n`,
  );
}

main();
