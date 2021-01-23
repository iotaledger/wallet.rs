const spawnSync = require('child_process').spawnSync
const path = require('path')
const fs = require('fs')
const pkg = require(path.join(__dirname, '../package.json'))
const getFileName = require('./utils').getFileName

let spawned = spawnSync('yarn', ['build:neon'], {
  cwd: path.join(__dirname, '..'),
  stdio: 'inherit'
})
if (spawned.status === 0) {
  const outputDir = path.resolve(__dirname, '../builds')
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir)
  }

  const outputFileName = getFileName(pkg)

  fs.copyFileSync(path.join(__dirname, '../native/index.node'), path.join(outputDir, outputFileName))
} else {
  console.error(`Failed to build neon module: ${spawned.error}`)
}
