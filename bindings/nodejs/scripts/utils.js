const getAbi = require('node-abi').getAbi

function getFileName (pkg) {
  const pkgName = pkg.name.replace(/^@\w+\//, '')
  const target = process.env.npm_config_target || process.versions.node
  const runtime = process.env.npm_config_runtime || 'node'
  const abi = getAbi(target, runtime)
  const platform = process.env.npm_config_platform || process.platform
  const arch = process.env.npm_config_arch || process.arch
  return `${pkgName}-v${pkg.version}-${runtime}-v${abi}-${platform}-${arch}.node`
}

module.exports = {
  getFileName
}
