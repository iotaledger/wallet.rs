const { resolve } = require('path');
const { spawnSync } = require('child_process');

// Passing "--prepack 'yarn build:neon'" causes problems on Windows, so this is a workaround

spawnSync('yarn', ['build:neon'], { stdio: 'inherit', cwd: resolve(__dirname, '../') });
