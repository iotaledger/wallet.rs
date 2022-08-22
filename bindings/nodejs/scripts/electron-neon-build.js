const { promisify } = require('util');
const moveArtifact = require('./move-artifact');
const build = promisify(require('electron-build-env'));

build(['npm', 'run', 'build:neon'], {
    electron: process.env.CURRENT_ELECTRON_VERSION,
}).then(() => {
    moveArtifact();
    process.exit(0)
});

