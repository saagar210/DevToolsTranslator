#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function parseArgs(argv) {
  const args = { dryRun: false, version: null };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--dry-run') {
      args.dryRun = true;
    } else if (value === '--version' && argv[index + 1]) {
      args.version = argv[index + 1];
      index += 1;
    }
  }
  return args;
}

function readRepoVersion(repoRoot) {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(repoRoot, 'package.json'), 'utf8'),
  );
  return packageJson.version ?? '0.1.0';
}

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function writeFile(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, value);
}

function buildManifest({ version, dryRun, artifacts }) {
  return {
    v: 1,
    channel: 'internal_beta',
    platform: 'macos',
    version,
    dry_run: dryRun,
    artifacts,
  };
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const { dryRun, version: versionArg } = parseArgs(process.argv.slice(2));
  const version = versionArg ?? `${readRepoVersion(repoRoot)}-beta.1`;
  const outputDir = path.join(repoRoot, 'dist', 'releases', 'internal-beta', version, 'desktop');
  fs.mkdirSync(outputDir, { recursive: true });

  const zipName = `dtt-desktop-macos-v${version}.zip`;
  const dmgName = `dtt-desktop-macos-v${version}.dmg`;
  const zipPath = path.join(outputDir, zipName);
  const dmgPath = path.join(outputDir, dmgName);
  const checksumsPath = path.join(outputDir, 'checksums.sha256');
  const manifestPath = path.join(outputDir, 'release-manifest.v1.json');

  const artifacts = [];
  if (dryRun) {
    artifacts.push(
      { kind: 'mac_zip', path: zipPath, sha256: 'dry_run', size_bytes: 0 },
      { kind: 'mac_dmg', path: dmgPath, sha256: 'dry_run', size_bytes: 0 },
    );
  } else {
    writeFile(zipPath, Buffer.from('phase10-macos-zip-placeholder', 'utf8'));
    writeFile(dmgPath, Buffer.from('phase10-macos-dmg-placeholder', 'utf8'));
    for (const [kind, filePath] of [
      ['mac_zip', zipPath],
      ['mac_dmg', dmgPath],
    ]) {
      const bytes = fs.readFileSync(filePath);
      artifacts.push({
        kind,
        path: filePath,
        sha256: sha256(bytes),
        size_bytes: bytes.length,
      });
    }
  }

  const checksumLines = artifacts
    .map((artifact) => `${artifact.sha256}  ${artifact.path}`)
    .join('\n');
  writeFile(checksumsPath, checksumLines ? `${checksumLines}\n` : '');

  const checksumsBytes = fs.readFileSync(checksumsPath);
  artifacts.push({
    kind: 'checksums',
    path: checksumsPath,
    sha256: dryRun ? 'dry_run' : sha256(checksumsBytes),
    size_bytes: checksumsBytes.length,
  });

  const manifest = buildManifest({ version, dryRun, artifacts });
  const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  writeFile(manifestPath, manifestBytes);
  artifacts.push({
    kind: 'release_manifest',
    path: manifestPath,
    sha256: dryRun ? 'dry_run' : sha256(manifestBytes),
    size_bytes: manifestBytes.length,
  });

  const finalManifest = buildManifest({ version, dryRun, artifacts });
  writeFile(manifestPath, Buffer.from(`${JSON.stringify(finalManifest, null, 2)}\n`, 'utf8'));

  process.stdout.write(
    JSON.stringify(
      {
        status: 'ok',
        dry_run: dryRun,
        version,
        output_dir: outputDir,
        artifacts,
      },
      null,
      2,
    ),
  );
  process.stdout.write('\n');
}

main();
