const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('Building WebAssembly module...');

// Create output directory if it doesn't exist
const outDir = path.join(__dirname, '..', 'pkg');
if (!fs.existsSync(outDir)) {
  fs.mkdirSync(outDir, { recursive: true });
}

// Build the WebAssembly module
try {
  // Build with wasm-pack
  execSync('wasm-pack build --target web --out-dir ../pkg --out-name clearcast-core -- --features wasm', {
    stdio: 'inherit',
    cwd: __dirname
  });

  // Copy package.json to pkg directory
  const pkg = require('../package.json');
  
  // Clean up package.json for wasm-pack output
  const wasmPkg = {
    name: pkg.name,
    version: pkg.version,
    description: pkg.description,
    main: 'clearcast_core.js',
    types: 'clearcast_core.d.ts',
    files: ['*'],
    repository: pkg.repository,
    author: pkg.author,
    license: pkg.license,
    bugs: pkg.bugs,
    homepage: pkg.homepage,
    keywords: pkg.keywords,
    sideEffects: false
  };

  fs.writeFileSync(
    path.join(outDir, 'package.json'),
    JSON.stringify(wasmPkg, null, 2)
  );

  console.log('WebAssembly build completed successfully!');
} catch (error) {
  console.error('Error building WebAssembly module:', error);
  process.exit(1);
}
