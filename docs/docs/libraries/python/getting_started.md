# Getting Started with IOTA Wallet Python binding

## Security
:::warning
It is not recommended to store passwords on the host's environment variables, or in the source code in a production setup. 
Please make sure you follow our [backup and security recommendations](https://chrysalis.docs.iota.org/guides/backup_security.html) for production use.
:::

## Installation
The easiest way to get python binding up and running is to leverage pre-built python libraries for linux, MacOs or Windows that can be installed to your python environment (3.6+) via `pip`. The binding is automagically generated using github [actions](https://github.com/iotaledger/wallet.rs/actions/workflows/python_binding_publish.yml).


You can download the latest artifacts for major python version using the  [nighly.link service](https://nightly.link/iotaledger/wallet.rs/workflows/python_binding_publish/develop).  
1. Download zip file for the given os and pyversion. 
2. Unpack wheel file (`.whl`).
3. Install it via `pip` by running the following command:

```bash
pip install <wheel_file>
```

Once it has been properly installed you can double-check the installation using `pip`:
```bash
pip list
```

The pip list should now include the `iota-wallet-python-binding`:
```plaintext
Package                    Version
-------------------------- -------
iota-wallet-python-binding 0.1.0
```

Once you python environment has installed the `iota-wallet-python-binding` you can start developing using the python binding.

## Usage
To use the `iota_wallet` you will need to add an import statement:  
```python
import iota_wallet
```
If you'd like more information on the `iota_wallet`, you can print the documentation using the following snippet:
```python
print(iota_wallet.__doc__)
```