import { execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const licensesDir = join(projectRoot, 'static', 'licenses');

// Ensure licenses directory exists
if (!existsSync(licensesDir)) {
  mkdirSync(licensesDir, { recursive: true });
}

console.log('Generating license information...\n');

// Generate npm licenses
console.log('1. Generating npm licenses...');
try {
  execSync('pnpm gen-licenses:npm', {
    cwd: projectRoot,
    stdio: 'inherit',
  });
  console.log('   npm licenses generated successfully.\n');
} catch (error) {
  console.error('   Failed to generate npm licenses:', error.message);
  // Create empty file as fallback
  writeFileSync(join(licensesDir, 'npm.json'), '{}');
}

// Generate Rust licenses
console.log('2. Generating Rust licenses...');
try {
  // Check if cargo-about is installed
  try {
    execSync('cargo about --version', { stdio: 'pipe' });
  } catch {
    console.log('   Installing cargo-about...');
    execSync('cargo install cargo-about', {
      cwd: projectRoot,
      stdio: 'inherit',
    });
  }

  execSync('pnpm gen-licenses:rust', {
    cwd: projectRoot,
    stdio: 'inherit',
  });
  console.log('   Rust licenses generated successfully.\n');
} catch (error) {
  console.error('   Failed to generate Rust licenses:', error.message);
  // Create empty file as fallback
  writeFileSync(join(licensesDir, 'rust.json'), '[]');
}

// Combine and format licenses
console.log('3. Combining license data...');
try {
  const npmLicensesPath = join(licensesDir, 'npm.json');
  const rustLicensesPath = join(licensesDir, 'rust.json');

  let npmLicenses = {};
  let rustLicenses = [];

  if (existsSync(npmLicensesPath)) {
    npmLicenses = JSON.parse(readFileSync(npmLicensesPath, 'utf-8'));
  }
  if (existsSync(rustLicensesPath)) {
    const content = readFileSync(rustLicensesPath, 'utf-8');
    try {
      const rustData = JSON.parse(content);
      // Extract crate information in a lightweight format
      if (rustData.crates && Array.isArray(rustData.crates)) {
        rustLicenses = rustData.crates.map((crate) => ({
          name: crate.package?.name || '',
          version: crate.package?.version || '',
          license: crate.license || crate.package?.license || 'Unknown',
          repository: crate.package?.repository || '',
          authors: (crate.package?.authors || []).join(', '),
        }));
      }
    } catch {
      rustLicenses = [];
    }
  }

  // Convert npm licenses to array format (exclude own package)
  const npmArray = Object.entries(npmLicenses)
    .filter(([key]) => !key.startsWith('hush-whisper@'))
    .map(([key, value]) => {
      const [name, version] = key.split('@').reduce(
        (acc, part, idx, arr) => {
          if (idx === arr.length - 1) {
            return [acc[0], part];
          }
          return [acc[0] ? `${acc[0]}@${part}` : part, acc[1]];
        },
        ['', ''],
      );
      return {
        name: name || key,
        version: version || '',
        license: value.licenses || 'Unknown',
        repository: value.repository || '',
        publisher: value.publisher || '',
      };
    });

  // Create combined output
  const combined = {
    generated: new Date().toISOString(),
    npm: npmArray,
    rust: rustLicenses,
  };

  writeFileSync(
    join(licensesDir, 'licenses.json'),
    JSON.stringify(combined, null, 2),
  );
  console.log('   Combined licenses saved to static/licenses/licenses.json\n');
} catch (error) {
  console.error('   Failed to combine licenses:', error.message);
}

console.log('License generation complete!');
