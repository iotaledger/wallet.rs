---
description: Getting started with the official IOTA Wallet Library Python binding.
image: /img/logo/iota_mark_light.png
keywords:
- Python
- install
- pip
- unpack
---
# Getting Started with IOTA Wallet Python Binding

## Security

:::warning
In a production setup, do not store passwords in the host's environment variables or in the source code. See our [backup and security recommendations](https://wiki.iota.org/chrysalis-docs/guides/backup_security) for production setups.
:::

## Requirements

[Python 3.x](https://www.python.org) & [pip](https://pypi.org/project/pip)

`Rust` and `Cargo`, to compile the binding. Install them [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Installation

- Go to `wallet.rs/bindings/python/native`

### Create a virtual environment and use it (optional)
- `python3 -m venv iota_wallet_venv`
- `source iota_wallet_venv/bin/activate`; Windows: `.\iota_wallet_venv\Scripts\activate`

### Install required dependencies and build the wheel
- `pip install -r requirements-dev.txt`
- `pip install .`

### Run examples
`python3 example/[example file]`

Example: 
- `python3 examples/0-create-account.py`

### To deactivate the virtual environment (optional)
- `deactivate`

## Getting Started

After you installed the library, you can create a `IotaWallet` instance and interact with it.

```python
from iota_wallet import IotaWallet, StrongholdSecretManager

# This example creates a new database and account

client_options = {
    'nodes': ['https://api.testnet.shimmer.network'],
}

# Shimmer coin type
coin_type = 4219

secret_manager = StrongholdSecretManager("wallet.stronghold", "some_hopefully_secure_password")

wallet = IotaWallet('./alice-database', client_options, coin_type, secret_manager)

# Store the mnemonic in the Stronghold snapshot, this only needs to be done once
account = wallet.store_mnemonic("flame fever pig forward exact dash body idea link scrub tennis minute " +
          "surge unaware prosper over waste kitten ceiling human knife arch situate civil")

account = wallet.create_account('Alice')
print(account)
```