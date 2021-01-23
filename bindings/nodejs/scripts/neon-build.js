const path = require('path')
const { spawnSync } = require('child_process')

spawnSync(
  path.join(__dirname, '../node_modules/.bin/neon'),
  ['build', '--release'],
	{ stdio: 'inherit' }
)
