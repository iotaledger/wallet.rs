---
description: Getting started with the official IOTA Wallet Library Software Python binding.
image: /img/logo/wallet_light.png
keywords:
- Python
- install
- pip
- unpack
- reference
---
# Getting Started with IOTA Wallet Python Binding

## Security

:::note

In a production setup, do not store passwords in the host's environment variables or in the source code. For reference, review our [backup and security recommendations](https://wiki.iota.org/chrysalis-docs/guides/backup_securityy) for production setups.
:::

## Installation

To get the python binding working, you need to leverage pre-built python libraries for linux, MacOs, or Windows. You can install these to your python environment (3.6+) using _pip_ . The binding is automatically generated using [GitHub actions](https://github.com/iotaledger/wallet.rs/actions/workflows/python_binding_publish.yml).

You can download the latest artifacts for a major python version using the [nighly.link service](https://nightly.link/iotaledger/wallet.rs/workflows/python_binding_publish/dev).  

1. Download zip file for the given os and pyversion. 
2. Unpack wheel file ( _.whl_ ).
3. Install it via _pip_ by running the following command:

```bash
pip install <wheel_file>
```

Once it has been installed, you can double-check the installation using _pip_ :

```bash
pip list
```

The pip list should now include the _iota-wallet-python-binding_:

```plaintext
Package                    Version
-------------------------- -------
iota-wallet-python-binding 0.1.0
```

Once your python environment has installed the `iota-wallet-python-binding`, you can start developing using the python binding.

## Usage

To use the _iota_wallet_ you will need to add an import statement:  

```python
import iota_wallet
```

If you'd like more information on the _iota_wallet_ , you can print the documentation using the following snippet:

```python
print(iota_wallet.__doc__)
```
