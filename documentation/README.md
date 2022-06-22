# Documentation

For a guidance on writing documentation, see the IOTA Wiki [contribution guidelines](https://wiki.iota.org/participate/contribute-to-wiki/welcome) and the IOTA Wiki [documentation template](https://github.com/iota-wiki/docs-template/tree/main/documentation).

## Tooling

The documentation is built using [Docusaurus 2](https://docusaurus.io/). The deployment is done through a centralized build from [IOTA WIKI](https://github.com/iota-community/iota-wiki). To run a local instance the [IOTA WIKI CLI](https://github.com/iota-community/iota-wiki-cli) is used.

### Prerequisites

- [Node.js v14.14+](https://nodejs.org/en/)
- [yarn](https://yarnpkg.com/getting-started/install)

### Installation

```console
yarn
```

This command installs all necessary dependencies.

### Local Development

```console
yarn start
```

This command starts a local, wiki themed development server and opens up a browser window. Most changes are reflected live without having to restart the server.

### Including .md file

```console
{@import <file path>}
```

Example:

```console
{@import ../../../../bindings/wasm/docs/api-reference.md}
```
