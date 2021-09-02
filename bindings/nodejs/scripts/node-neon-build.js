const { resolve } = require('path');
const { spawnSync } = require('child_process');
const moveArtifact = require('./move-artifact');

// Passing "--prepack 'npm run build:neon'" causes problems on Windows, so this is a workaround

const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm';

const { status } = spawnSync(npm, ['run', 'build:neon'], {
    stdio: 'inherit',
    cwd: resolve(__dirname, '../'),
});

if (status === null) {
    process.exit(1);
} else if (status > 0) {
    process.exit(status);
}

moveArtifact();
