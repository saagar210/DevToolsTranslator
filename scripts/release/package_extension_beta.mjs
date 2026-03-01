#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';

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

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function readVersion(repoRoot, explicitVersion) {
  if (explicitVersion) {
    return explicitVersion;
  }
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(repoRoot, 'apps', 'extension-mv3', 'package.json'), 'utf8'),
  );
  return `${packageJson.version ?? '0.1.0'}-beta.1`;
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const extensionRoot = path.join(repoRoot, 'apps', 'extension-mv3');
  const distDir = path.join(extensionRoot, 'dist');
  const { dryRun, version: versionArg } = parseArgs(process.argv.slice(2));
  const version = readVersion(repoRoot, versionArg);

  const outputDir = path.join(repoRoot, 'dist', 'releases', 'internal-beta', version, 'extension');
  fs.mkdirSync(outputDir, { recursive: true });

  const zipName = `dtt-extension-mv3-v${version}.zip`;
  const zipPath = path.join(outputDir, zipName);
  const checksumsPath = path.join(outputDir, 'checksums.sha256');
  const manifestPath = path.join(outputDir, 'release-manifest.v1.json');

  let zipSha = 'dry_run';
  let zipSize = 0;

  if (!dryRun) {
    if (!fs.existsSync(distDir)) {
      throw new Error(
        `Extension dist not found at ${distDir}. Run "pnpm --filter @dtt/extension build" first.`,
      );
    }
    execFileSync('zip', ['-rq', zipPath, '.'], { cwd: distDir, stdio: 'pipe' });
    const bytes = fs.readFileSync(zipPath);
    zipSha = sha256(bytes);
    zipSize = bytes.length;
  }

  const artifacts = [
    {
      kind: 'extension_zip',
      path: zipPath,
      sha256: zipSha,
      size_bytes: zipSize,
    },
  ];
  const checksumLine = `${zipSha}  ${zipPath}\n`;
  fs.writeFileSync(checksumsPath, checksumLine, 'utf8');
  artifacts.push({
    kind: 'checksums',
    path: checksumsPath,
    sha256: dryRun ? 'dry_run' : sha256(Buffer.from(checksumLine, 'utf8')),
    size_bytes: Buffer.byteLength(checksumLine),
  });

  const manifest = {
    v: 1,
    channel: 'internal_beta',
    target: 'chrome_mv3',
    version,
    dry_run: dryRun,
    artifacts,
  };
  writeJson(manifestPath, manifest);
  const manifestBytes = fs.readFileSync(manifestPath);
  artifacts.push({
    kind: 'release_manifest',
    path: manifestPath,
    sha256: dryRun ? 'dry_run' : sha256(manifestBytes),
    size_bytes: manifestBytes.length,
  });
  writeJson(manifestPath, { ...manifest, artifacts });

  process.stdout.write(
    JSON.stringify(
      {
        status: 'ok',
        dry_run: dryRun,
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
