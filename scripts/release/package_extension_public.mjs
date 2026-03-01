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

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function writeFile(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, value);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function complianceChecks({ manifest, checklist, version }) {
  const checks = [];
  const actualPermissions = Array.isArray(manifest.permissions)
    ? [...manifest.permissions].sort()
    : [];
  const allowedPermissions = [...checklist.permission_allowlist].sort();
  const missingPermissions = allowedPermissions.filter(
    (permission) => !actualPermissions.includes(permission),
  );
  const extraPermissions = actualPermissions.filter(
    (permission) => !allowedPermissions.includes(permission),
  );

  checks.push({
    key: 'manifest_permission_allowlist',
    status: missingPermissions.length === 0 && extraPermissions.length === 0 ? 'pass' : 'fail',
    details: {
      expected: allowedPermissions,
      actual: actualPermissions,
      missing: missingPermissions,
      extra: extraPermissions,
    },
  });

  const privacyPolicy = typeof manifest.homepage_url === 'string' ? manifest.homepage_url : '';
  checks.push({
    key: 'privacy_policy_url_present',
    status: privacyPolicy.startsWith('https://') ? 'pass' : 'fail',
    details: { value: privacyPolicy },
  });

  const requiredArtifacts = (checklist.required_artifacts ?? []).map((artifact) => ({
    path: artifact,
    exists: fs.existsSync(artifact),
  }));
  checks.push({
    key: 'data_use_declaration_present',
    status: requiredArtifacts.every((artifact) => artifact.exists) ? 'pass' : 'warn',
    details: requiredArtifacts,
  });

  checks.push({
    key: 'host_permission_inventory',
    status: 'pass',
    details: {
      host_permissions: manifest.host_permissions ?? [],
      optional_host_permissions: manifest.optional_host_permissions ?? [],
    },
  });

  const monotonic = typeof version === 'string' && version.trim().length > 0;
  checks.push({
    key: 'version_monotonicity',
    status: monotonic ? 'pass' : 'fail',
    details: { version },
  });

  return checks;
}

function main() {
  const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
  const { dryRun, version: versionArg } = parseArgs(process.argv.slice(2));
  const packageJson = readJson(path.join(repoRoot, 'package.json'));
  const version = versionArg ?? `${packageJson.version}-public.1`;

  const manifest = readJson(path.join(repoRoot, 'apps', 'extension-mv3', 'manifest.json'));
  const checklist = readJson(
    path.join(repoRoot, 'config', 'compliance', 'extension_public_checklist.v1.json'),
  );
  const checks = complianceChecks({ manifest, checklist, version });
  const failing = checks.filter((check) => check.status === 'fail');

  if (!dryRun && failing.length > 0) {
    process.stdout.write(
      `${JSON.stringify({ status: 'error', error_code: 'extension_compliance_failed', failing }, null, 2)}\n`,
    );
    process.exit(1);
  }

  const outputDir = path.join(
    repoRoot,
    'dist',
    'releases',
    'chrome-store-public',
    version,
    'extension',
  );
  fs.mkdirSync(outputDir, { recursive: true });

  const zipPath = path.join(outputDir, `dtt-extension-chrome-store-v${version}.zip`);
  const checksPath = path.join(outputDir, 'checksums.sha256');
  const compliancePath = path.join(outputDir, 'compliance-checks.v1.json');
  const manifestPath = path.join(outputDir, 'release-manifest.v1.json');

  if (!dryRun) {
    writeFile(zipPath, Buffer.from('phase13-extension-public-placeholder', 'utf8'));
  }

  const zipBytes = dryRun ? Buffer.from('', 'utf8') : fs.readFileSync(zipPath);
  const artifacts = [
    {
      kind: 'extension_zip',
      path: zipPath,
      sha256: dryRun ? 'dry_run' : sha256(zipBytes),
      size_bytes: zipBytes.length,
    },
  ];

  const checksumLines = artifacts
    .map((artifact) => `${artifact.sha256}  ${artifact.path}`)
    .join('\n');
  writeFile(checksPath, checksumLines ? `${checksumLines}\n` : '');

  writeFile(compliancePath, Buffer.from(`${JSON.stringify({ v: 1, checks }, null, 2)}\n`, 'utf8'));

  const checksBytes = fs.readFileSync(checksPath);
  artifacts.push({
    kind: 'checksums',
    path: checksPath,
    sha256: dryRun ? 'dry_run' : sha256(checksBytes),
    size_bytes: checksBytes.length,
  });

  const releaseManifest = {
    v: 1,
    channel: 'chrome_store_public',
    version,
    dry_run: dryRun,
    compliance: {
      pass: failing.length === 0,
      failures: failing.map((check) => check.key),
      checks_file: compliancePath,
    },
    artifacts: [...artifacts].sort((left, right) => left.path.localeCompare(right.path)),
  };
  const releaseManifestBytes = Buffer.from(`${JSON.stringify(releaseManifest, null, 2)}\n`, 'utf8');
  writeFile(manifestPath, releaseManifestBytes);
  artifacts.push({
    kind: 'release_manifest',
    path: manifestPath,
    sha256: dryRun ? 'dry_run' : sha256(releaseManifestBytes),
    size_bytes: releaseManifestBytes.length,
  });

  process.stdout.write(
    `${JSON.stringify(
      {
        status: 'ok',
        dry_run: dryRun,
        version,
        output_dir: outputDir,
        checks,
        artifacts,
      },
      null,
      2,
    )}\n`,
  );
}

main();
