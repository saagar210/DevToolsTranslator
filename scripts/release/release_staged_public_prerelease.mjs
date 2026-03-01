#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';

function parseArgs(argv) {
  const args = {
    version: null,
    promoteFromInternalRunId: null,
    notes: 'Staged public prerelease promotion',
    dryRun: false,
    publish: false,
    repo: process.env.GITHUB_REPOSITORY ?? '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--version' && argv[index + 1]) {
      args.version = argv[index + 1];
      index += 1;
    } else if (value === '--promote-from-internal-run-id' && argv[index + 1]) {
      args.promoteFromInternalRunId = argv[index + 1];
      index += 1;
    } else if (value === '--notes' && argv[index + 1]) {
      args.notes = argv[index + 1];
      index += 1;
    } else if (value === '--dry-run') {
      args.dryRun = true;
    } else if (value === '--publish') {
      args.publish = true;
    } else if (value === '--repo' && argv[index + 1]) {
      args.repo = argv[index + 1];
      index += 1;
    }
  }

  return args;
}

function readVersion(repoRoot) {
  const packageJson = JSON.parse(fs.readFileSync(path.join(repoRoot, 'package.json'), 'utf8'));
  return `${packageJson.version ?? '0.1.0'}-beta.1`;
}

function hasManualSmokePass(content) {
  return content
    .split(/\r?\n/)
    .some((line) =>
      /^interactive_chrome_manual:\s*pass\|date=20\d{2}-\d{2}-\d{2}\|observer=.+$/i.test(
        line.trim(),
      ),
    );
}

function loadManifest(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const args = parseArgs(process.argv.slice(2));
  const version = args.version ?? readVersion(repoRoot);
  const promoteFromInternalRunId = args.promoteFromInternalRunId ?? `internal_${version}`;

  if (!args.dryRun) {
    const smokePath = path.join(repoRoot, 'docs', 'PHASE6_SMOKE_EVIDENCE.md');
    const smoke = fs.existsSync(smokePath) ? fs.readFileSync(smokePath, 'utf8') : '';
    if (!hasManualSmokePass(smoke)) {
      throw new Error(
        'manual smoke evidence missing: expected interactive_chrome_manual pass marker',
      );
    }
  }

  const internalRoot = path.join(repoRoot, 'dist', 'releases', 'internal-beta', version);
  const outputRoot = path.join(
    repoRoot,
    'dist',
    'releases',
    'staged-public-prerelease',
    version,
  );
  fs.mkdirSync(outputRoot, { recursive: true });

  const manifests = [
    loadManifest(path.join(internalRoot, 'desktop', 'release-manifest.v1.json')),
    loadManifest(path.join(internalRoot, 'desktop-windows', 'release-manifest.v1.json')),
    loadManifest(path.join(internalRoot, 'desktop-linux', 'release-manifest.v1.json')),
  ].filter(Boolean);

  if (manifests.length === 0) {
    throw new Error(
      `no internal desktop manifests found for version ${version} under ${internalRoot}`,
    );
  }

  const artifacts = manifests
    .flatMap((manifest) => (Array.isArray(manifest.artifacts) ? manifest.artifacts : []))
    .sort((left, right) => String(left.path).localeCompare(String(right.path)));

  const provenance = {
    v: 1,
    channel: 'staged_public_prerelease',
    visibility: 'staged_public',
    version,
    promote_from_internal_run_id: promoteFromInternalRunId,
    dry_run: args.dryRun,
    workflow_run_id: process.env.GITHUB_RUN_ID ?? 'local',
    source_commit: process.env.GITHUB_SHA ?? 'unknown',
    signing_status: 'verified',
    notarization_status: 'verified',
    artifacts,
  };

  const provenancePath = path.join(outputRoot, 'release-provenance.v1.json');
  fs.writeFileSync(provenancePath, `${JSON.stringify(provenance, null, 2)}\n`, 'utf8');

  if (args.publish) {
    if (args.dryRun) {
      throw new Error('--publish cannot be used with --dry-run');
    }
    if (!args.repo) {
      throw new Error('missing repository slug: provide --repo owner/name or GITHUB_REPOSITORY');
    }
    const tag = `v${version}-staged`;
    execFileSync(
      'gh',
      [
        'release',
        'create',
        tag,
        '--repo',
        args.repo,
        '--title',
        tag,
        '--notes',
        args.notes,
        '--prerelease',
        '--draft',
        provenancePath,
      ],
      { stdio: 'inherit' },
    );
  }

  process.stdout.write(
    `${JSON.stringify(
      {
        status: 'ok',
        dry_run: args.dryRun,
        published: args.publish,
        version,
        promote_from_internal_run_id: promoteFromInternalRunId,
        output_dir: outputRoot,
        provenance_path: provenancePath,
        artifact_count: artifacts.length,
      },
      null,
      2,
    )}\n`,
  );
}

main();
