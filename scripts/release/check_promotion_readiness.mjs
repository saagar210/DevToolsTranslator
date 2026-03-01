#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function parseArgs(argv) {
  const args = {
    version: null,
    channel: 'staged_public_prerelease',
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--version' && argv[index + 1]) {
      args.version = argv[index + 1];
      index += 1;
    } else if (value === '--channel' && argv[index + 1]) {
      args.channel = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function hasManualSmokePass(filePath) {
  if (!fs.existsSync(filePath)) return false;
  const content = fs.readFileSync(filePath, 'utf8');
  return content
    .split('\n')
    .map((line) => line.trim())
    .some((line) =>
      /^interactive_chrome_manual:\s*pass\|date=\d{4}-\d{2}-\d{2}\|observer=.+$/i.test(line),
    );
}

function exists(filePath) {
  return fs.existsSync(filePath);
}

function hasEnv(name) {
  const value = process.env[name];
  return typeof value === 'string' && value.trim().length > 0;
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const args = parseArgs(process.argv.slice(2));
  if (!args.version) {
    process.stderr.write('Missing required --version\n');
    process.exit(2);
  }

  const checks = [];
  const blockers = [];

  const smokeFile = path.join(repoRoot, 'docs', 'PHASE6_SMOKE_EVIDENCE.md');
  const manualSmokePass = hasManualSmokePass(smokeFile);
  checks.push({
    key: 'manual_smoke_marker',
    status: manualSmokePass ? 'pass' : 'fail',
    details: { file: smokeFile },
  });
  if (!manualSmokePass) blockers.push('manual_smoke_missing');

  const requiredManifests = [
    path.join(repoRoot, 'dist', 'releases', 'internal-beta', args.version, 'desktop', 'release-manifest.v1.json'),
    path.join(
      repoRoot,
      'dist',
      'releases',
      'internal-beta',
      args.version,
      'desktop-windows',
      'release-manifest.v1.json',
    ),
    path.join(
      repoRoot,
      'dist',
      'releases',
      'internal-beta',
      args.version,
      'desktop-linux',
      'release-manifest.v1.json',
    ),
    path.join(
      repoRoot,
      'dist',
      'releases',
      'chrome-store-public',
      args.version,
      'extension',
      'release-manifest.v1.json',
    ),
  ];
  const missingManifests = requiredManifests.filter((filePath) => !exists(filePath));
  checks.push({
    key: 'release_manifests_present',
    status: missingManifests.length === 0 ? 'pass' : 'fail',
    details: {
      required_count: requiredManifests.length,
      missing: missingManifests,
    },
  });
  if (missingManifests.length > 0) blockers.push('release_manifest_missing');

  const cwsEnv = ['CWS_CLIENT_ID', 'CWS_CLIENT_SECRET', 'CWS_REFRESH_TOKEN', 'CWS_EXTENSION_ID'];
  const missingCwsEnv = cwsEnv.filter((name) => !hasEnv(name));
  checks.push({
    key: 'cws_credentials_present',
    status: missingCwsEnv.length === 0 ? 'pass' : 'fail',
    details: {
      missing: missingCwsEnv,
    },
  });
  if (missingCwsEnv.length > 0) blockers.push('cws_credentials_missing');

  const updaterSignatureSet = hasEnv('UPDATER_SIGNATURE');
  checks.push({
    key: 'updater_signature_present',
    status: updaterSignatureSet ? 'pass' : 'fail',
    details: {
      channel: args.channel,
    },
  });
  if (!updaterSignatureSet) blockers.push('updater_signature_missing');

  const result = {
    v: 1,
    version: args.version,
    channel: args.channel,
    status: blockers.length === 0 ? 'ready' : 'blocked',
    blockers,
    checks,
    checked_at_ms: Date.now(),
  };

  const outputDir = path.join(repoRoot, 'dist', 'releases', 'readiness', args.version);
  fs.mkdirSync(outputDir, { recursive: true });
  const outputPath = path.join(outputDir, `promotion-readiness-${args.channel}.v1.json`);
  fs.writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');

  process.stdout.write(`${JSON.stringify({ ...result, report_path: outputPath }, null, 2)}\n`);
  process.exit(blockers.length === 0 ? 0 : 1);
}

main();
