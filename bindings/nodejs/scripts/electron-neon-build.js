const { promisify } = require('util');
const build = promisify(require('electron-build-env'));
const moveArtifact = require('./move-artifact');

build(['npm', 'run', 'build:neon'], {
    electron: process.env.CURRENT_ELECTRON_VERSION,
}).then(() => {
    moveArtifact();
    process.exit(0);
});
