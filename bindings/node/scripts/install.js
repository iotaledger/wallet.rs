const path = require('path')
const fs = require('fs')
const home = require('os').homedir
const pkg = require(path.join(__dirname, '../package.json'))
const getFileName = require('./utils').getFileName

const crypto = require('crypto')
const get = require('simple-get')
const mkdirp = require('mkdirp-classic')
const pump = require('pump')

const tagPrefix = 'nodejs-binding-v'

pkg.repository.url = 'https://github.com/iotaledger/entangled-node'
pkg.version = '0.5.1'
pkg.name = '@iota/entangled-node'

function npmCache () {
  var env = process.env
  return env.npm_config_cache || (env.APPDATA ? path.join(env.APPDATA, 'npm-cache') : path.join(home(), '.npm'))
}

function iotaCache () {
  return path.join(npmCache(), '.iota')
}

function cachedIotaPath (url) {
  var digest = crypto.createHash('md5').update(url).digest('hex').slice(0, 6)
  return path.join(iotaCache(), digest + '-' + path.basename(url).replace(/[^a-zA-Z0-9.]+/g, '-'))
}

function getTempFilePath (cached) {
  return cached + '.' + process.pid + '-' + Math.random().toString(16).slice(2) + '.tmp'
}

function download (downloadUrl, cb) {
  var cachedPath = cachedIotaPath(downloadUrl)
  var tempFilePath = getTempFilePath(cachedPath)

  ensureCacheDir(function (err) {
    if (err) return onerror(err)

    fs.access(cachedPath, fs.R_OK | fs.W_OK, function (err) {
      if (!(err && err.code === 'ENOENT')) {
        return finish('cache')
      }

      var reqOpts = { url: downloadUrl }

      console.log(reqOpts)
      var req = get(reqOpts, function (err, res) {
        if (err) return onerror(err)
        if (res.statusCode !== 200) return onerror(`got status code ${res.statusCode} requesting ${downloadUrl}`)
        mkdirp(iotaCache(), function () {
          pump(res, fs.createWriteStream(tempFilePath), function (err) {
            if (err) return onerror(err)
            fs.rename(tempFilePath, cachedPath, function (err) {
              if (err) return cb(err)
              finish('download')
            })
          })
        })
      })

      req.setTimeout(30 * 1000, function () {
        req.abort()
      })
    })

    function finish(source) {
      fs.copyFileSync(cachedPath, path.join(__dirname, '../native/index.node'))
      cb(null, `got file from ${source}`)
    }

    function onerror (err) {
      fs.unlink(tempFilePath, function () {
        cb(err || 'error')
      })
    }
  })

  function ensureCacheDir (cb) {
    var cacheFolder = iotaCache()
    fs.access(cacheFolder, fs.R_OK | fs.W_OK, function (err) {
      if (err && err.code === 'ENOENT') {
        return makeCacheDir()
      }
      cb(err)
    })

    function makeCacheDir () {
      mkdirp(cacheFolder, cb)
    }
  }
}

function combineURLs(baseURL, relativeURL) {
  return baseURL.replace(/\/+$/, '') + '/' + relativeURL.replace(/^\/+/, '')
}

const downloadUrl = combineURLs(pkg.repository.url, `releases/download/${tagPrefix}${pkg.version}/${getFileName(pkg)}`)
download(downloadUrl.toString(), (err, data) => {
  console.log(err, data)
  if (err) {
    console.error(`Failed to download installation. Error: ${err}`)
    process.exit(1)
  }
})
