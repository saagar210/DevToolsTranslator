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
  const packageJson = JSON.parse(fs.readFileSync(path.join(repoRoot, 'package.json'), 'utf8'));
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
    platform: 'linux',
    version,
    dry_run: dryRun,
    artifacts,
  };
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const { dryRun, version: versionArg } = parseArgs(process.argv.slice(2));
  const version = versionArg ?? `${readRepoVersion(repoRoot)}-beta.1`;
  const outputDir = path.join(repoRoot, 'dist', 'releases', 'internal-beta', version, 'desktop-linux');
  fs.mkdirSync(outputDir, { recursive: true });

  const appImageName = `dtt-desktop-linux-v${version}.AppImage`;
  const debName = `dtt-desktop-linux-v${version}.deb`;
  const tarName = `dtt-desktop-linux-v${version}.tar.gz`;
  const appImagePath = path.join(outputDir, appImageName);
  const debPath = path.join(outputDir, debName);
  const tarPath = path.join(outputDir, tarName);
  const checksumsPath = path.join(outputDir, 'checksums.sha256');
  const manifestPath = path.join(outputDir, 'release-manifest.v1.json');

  const artifacts = [];
  if (dryRun) {
    artifacts.push(
      { kind: 'linux_app_image', path: appImagePath, sha256: 'dry_run', size_bytes: 0 },
      { kind: 'linux_deb', path: debPath, sha256: 'dry_run', size_bytes: 0 },
      { kind: 'linux_tar_gz', path: tarPath, sha256: 'dry_run', size_bytes: 0 },
    );
  } else {
    writeFile(appImagePath, Buffer.from('phase11-linux-appimage-placeholder', 'utf8'));
    writeFile(debPath, Buffer.from('phase11-linux-deb-placeholder', 'utf8'));
    writeFile(tarPath, Buffer.from('phase11-linux-targz-placeholder', 'utf8'));
    for (const [kind, filePath] of [
      ['linux_app_image', appImagePath],
      ['linux_deb', debPath],
      ['linux_tar_gz', tarPath],
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

  artifacts.sort((left, right) => left.path.localeCompare(right.path));
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

  artifacts.sort((left, right) => left.path.localeCompare(right.path));
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
