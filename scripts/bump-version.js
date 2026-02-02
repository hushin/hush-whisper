#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');

const FILES = [
  'package.json',
  'src-tauri/Cargo.toml',
  'src-tauri/tauri.conf.json'
];

function getCurrentVersion() {
  const packageJson = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf-8'));
  return packageJson.version;
}

function bumpVersion(version, type) {
  const [major, minor, patch] = version.split('.').map(Number);

  switch (type) {
    case 'major':
      return `${major + 1}.0.0`;
    case 'minor':
      return `${major}.${minor + 1}.0`;
    case 'patch':
      return `${major}.${minor}.${patch + 1}`;
    default:
      throw new Error(`Invalid bump type: ${type}. Use major, minor, or patch`);
  }
}

function updateFile(filePath, oldVersion, newVersion) {
  const fullPath = join(rootDir, filePath);
  let content = readFileSync(fullPath, 'utf-8');

  if (filePath.endsWith('.toml')) {
    content = content.replace(
      /^version = ".*"$/m,
      `version = "${newVersion}"`
    );
  } else if (filePath.endsWith('.json')) {
    const json = JSON.parse(content);
    json.version = newVersion;
    content = JSON.stringify(json, null, 2) + '\n';
  }

  writeFileSync(fullPath, content, 'utf-8');
  console.log(`✓ ${filePath}: ${oldVersion} → ${newVersion}`);
}

function main() {
  const bumpType = process.argv[2] || 'patch';

  if (!['major', 'minor', 'patch'].includes(bumpType)) {
    console.error('Usage: pnpm bump [major|minor|patch]');
    console.error('Example: pnpm bump patch');
    process.exit(1);
  }

  const oldVersion = getCurrentVersion();
  const newVersion = bumpVersion(oldVersion, bumpType);

  console.log(`\nBumping version: ${oldVersion} → ${newVersion} (${bumpType})\n`);

  FILES.forEach(file => updateFile(file, oldVersion, newVersion));

  console.log(`\n✨ Version bumped successfully!`);
  console.log(`\nNext steps:`);
  console.log(`  git add -A`);
  console.log(`  git commit -m "chore: bump version to ${newVersion}"`);
  console.log(`  git tag v${newVersion}`);
  console.log(`  git push && git push --tags`);
  console.log(`\nTo create a release after building:`);
  console.log(`  pnpm tauri build`);
  console.log(`  gh release create v${newVersion} \\`);
  console.log(`    "src-tauri/target/release/bundle/msi/hush-whisper_${newVersion}_x64_ja-JP.msi" \\`);
  console.log(`    "src-tauri/target/release/bundle/nsis/hush-whisper_${newVersion}_x64-setup.exe" \\`);
  console.log(`    --title "v${newVersion}" \\`);
  console.log(`    --notes "Release v${newVersion}"`);
}

main();
