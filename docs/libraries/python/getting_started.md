# Getting Started with IOTA Wallet Python binding

## Installation
Easiest way how to get python binding up and running is to leverage pre-built python libraries for linux/macos/windows that can be installed to your python environment via `pip`. The binding is automagically generated using github [actions](https://github.com/iotaledger/wallet.rs/actions/workflows/python_binding_publish.yml).

The latest artifacts for major python versions can be also grabbed using [nighly.link service](https://nightly.link/iotaledger/wallet.rs/workflows/python_binding_publish/develop). Download zip file for the given os and pyversion, unpack wheel file (`.whl`) and install it via `pip`:

```bash
pip install <wheel_file>
```

Once it has been properly installed you can double check it using `pip`:
```bash
pip list
```

You should see the similar output:
```plaintext
Package                    Version
-------------------------- -------
iota-wallet-python-binding 0.1.0
```
Once installed in the given python environment you are all set and can start hacking using python binding!

## Usage

```python
import iota_wallet
print(iota_wallet.__doc__)
```
