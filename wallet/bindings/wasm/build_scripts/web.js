const path = require('path')
const fs = require('fs')
const { lintAll } = require('./lints')
const generatePackage = require('./utils/generatePackage');

const rustPackageName = "iota_wallet_wasm";
const wasmFilename = "iota_wallet_wasm_bg.wasm";

const RELEASE_FOLDER = path.join(__dirname, '../web/wasm/');
const entryFilePath = path.join(RELEASE_FOLDER, rustPackageName + '.js');
const entryFile = fs.readFileSync(entryFilePath).toString();

lintAll(entryFile);

let changedFile = entryFile
    // Comment out generated code as a workaround for webpack (does not recognise import.meta).
    // Regex to avoid hard-coding 'wallet_wasm_bg.wasm'.
    .replace(
        /input = new URL\((.*), import\.meta\.url\);/i,
        "// input = new URL($1, import.meta.url);"
    )
    // Rename original init function, because we want to use the name for our own function.
    .replace(
        "async function init(input) {",
        "async function initWasm(input) {"
    )
    .replace(
        "init.__wbindgen_wasm_module = module;",
        "initWasm.__wbindgen_wasm_module = module;"
    )
    // Create an init function which imports the wasm file.
    .replace(
        "export default init;",
        "let __initializedIotaWasm = false\r\n\r\nexport function init(path) {\r\n    if (__initializedIotaWasm) {\r\n        return Promise.resolve(wasm)\r\n    }\r\n    return initWasm(path || \'" + wasmFilename + "\').then(() => {\r\n        __initializedIotaWasm = true\r\n        return wasm\r\n    })\r\n}\r\n"
    );

fs.writeFileSync(
    entryFilePath,
    changedFile
);

const entryFilePathTs = path.join(RELEASE_FOLDER, rustPackageName + '.d.ts');
const entryFileTs = fs.readFileSync(entryFilePathTs).toString();
// Replace the init function in the ts file.
let changedFileTs = entryFileTs.replace(
    "/**\n* If `module_or_path` is {RequestInfo} or {URL}, makes a request and\n* for everything else, calls `WebAssembly.instantiate` directly.\n*\n* @param {InitInput | Promise<InitInput>} module_or_path\n*\n* @returns {Promise<InitOutput>}\n*/\nexport default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;",
    "\/**\r\n* Loads the Wasm file so the lib can be used, relative path to Wasm file\r\n* @param {string | undefined} path\r\n*\/\r\nexport function init(path?: string): Promise<void>;"
);
fs.writeFileSync(
    entryFilePathTs,
    changedFileTs
);

const newPackage = generatePackage({
    module: 'lib/index.js',
    types: 'lib/index.d.ts',
});

fs.writeFileSync(path.join(RELEASE_FOLDER + "../", 'package.json'), JSON.stringify(newPackage, null, 2));

// Export the Wasm init() function from `index.ts.
const indexFile = path.join(__dirname, "..", "out", "lib", "index.ts");
fs.writeFileSync(indexFile, "// @ts-ignore\nexport { init } from '../wasm/iota_wallet_wasm';", { flag: 'a' });
