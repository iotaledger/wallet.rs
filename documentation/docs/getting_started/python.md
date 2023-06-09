---
description: Getting started with the official IOTA Wallet Library Software Python binding.
image: /img/logo/logo_dark.svg
keywords:
- Python
- install
- pip
- unpack
- getting started
---
# Getting Started with Python

## Security

:::note

In a production setup, do not store passwords in the host's environment variables or in the source code. For reference, review our [backup and security recommendations](https://wiki.iota.org/introduction/how_tos/backup_security/) for production setups.
:::

## Requirements

* [Python 3.x](https://www.python.org).
* [pip](https://pypi.org/project/pip).

## Installation

### Install prebuild libraries

To get the python binding working, you need to leverage pre-built python libraries for linux, MacOs, or Windows. You can install these to your python environment (3.6+) using _pip_ . The binding is automatically generated using [GitHub actions](https://github.com/iotaledger/wallet.rs/actions/workflows/python_binding_publish.yml).

You can download the latest artifacts for a major python version using the [nighly.link service](https://nightly.link/iotaledger/wallet.rs/workflows/python_binding_publish/production).  

1. Download zip file for the given os and pyversion. 
2. Unpack wheel file ( _.whl_ ).
3. Install it via _pip_ by running the following command:

```bash
pip install <wheel_file>
```

### Install from source

### Additional Requirements

* [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
* (for Linux only) `libudev`. You can install it with `apt install libudev-dev`.

#### Clone the Repository

You can clone the wallet.rs client library by running the following command:

```bash
git clone -b production https://github.com/iotaledger/wallet.rs
```

#### Change to the Python Binding Directory

After you have cloned the repository, you should change directory to `wallet.rs/bindings/python/native`. You can do so by running the following command:

```bash
cd wallet.rs/bindings/python/native
```

#### Install the Required Dependencies and Build the Wheel

Install and run maturin:

```bash
pip3 install maturin
maturin develop
maturin build --manylinux off
```
The wheel file is now created in `bindings/python/native/target/wheels`. You can install it with:

```bash
pip3 install [THE_BUILT_WHEEL_FILE]
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

If you'd like more information on the _iota_wallet_, you can print the documentation using the following snippet:

```python
print(iota_wallet.__doc__)
```
