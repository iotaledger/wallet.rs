## Prerequisites

Before you can run the examples, please refer to the [Rust Getting Started guide](./../getting_started/rust) to install
the library.

### Clone the Repository

To run the rust examples, you will first need to clone the repository. You can do so by running the following command:

```bash
git clone git@github.com:iotaledger/wallet.rs.git
```

### Set Up Your .env file

After you have [cloned the project](#clone-the-repository), you should:

1. Move into the project directory by running the following command:

```bash
cd wallet.rs
```

2. Create your `.env` file by making a copy of the `.env.example` file by running the following command:

```bash
cp .env.example .env
```

You should update the `NODE_URL` and `FAUCET_URL` values to match the [Hornet node](#hornet-node) you want to use.

## Run Code Examples

The wallet.rs library has numerous [examples](https://github.com/iotaledger/wallet.rs/tree/develop/examples)
you can run to get acquainted with the library. After you have followed the instructions to
[install the library](./../getting_started/rust#install-the-library), you can run any example with the following
command from the `examples` directory:

```bash
cargo run --example 0_generate_addresses --release
```

## Examples List

You can replace the `0_generate_addresses` by any other example from the [Rust examples directory](https://github.com/iotaledger/wallet.rs/tree/develop/examples).

You can get a full list of examples by running the following command:

```bash
cargo run --example
```
