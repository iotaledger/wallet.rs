const { resolve } = require('path');
const { spawnSync } = require('child_process');

// Based on https://github.com/prebuild/prebuild/blob/master/strip.js

const binaryPath = resolve(__dirname, '../index.node');
const stripArgs = process.platform === 'darwin' ? '-Sx' : '--strip-all';
spawnSync('strip', [stripArgs, binaryPath], { stdio: 'inherit' });
