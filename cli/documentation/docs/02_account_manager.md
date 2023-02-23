# Account Manager Interface

The Account Manager Interface is evaluated through the Command Line Interface of the `wallet` binary, once per
execution.

It is responsible for the creation and management of the wallet and its accounts.

## Commands

### `./wallet`

Starts the wallet without a specified account:
- If the wallet has only one account, it will be used;
- If the wallet has more than one account, a selector will be shown to decide which account to use.

The wallet needs to be initialised (`init` command) and with at least one account (`new` command).

#### Example

```sh
./wallet
```

### `./wallet [account]`

Starts the wallet with a specified account;

The wallet needs to be initialised (`init` command).

#### Example

```sh
./wallet main
```

### `./wallet backup`

Creates a stronghold backup file.

#### Parameters

| Name    | Optional  | Example           |
| ------- | --------- | ----------------- |
| `path`  | ✘         | backup.stronghold |

#### Example

Create a stronghold backup file.
```sh
./wallet backup backup.stronghold
```

### `./wallet change-password`

Changes the stronghold password.

#### Example

Change the stronghold password.
```sh
./wallet change-password
```

### `./wallet help`

Displays the account manager interface usage and exits.

#### Example

```sh
./wallet help
```

### `./wallet init`

Initialises the wallet by creating a [stronghold](https://github.com/iotaledger/stronghold.rs) file from a provided or generated mnemonic.

The wallet can only be initialised once.

When just initialised, the wallet has no account yet, use the `new` command to create one.

#### Parameters

| Name        | Optional    | Default                | Example                                                                                                                                                                             |
| ----------- | ----------- |----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mnemonic`  | ✓           | Randomly generated     | "aunt middle impose faith ramp kid olive good practice motor grab ready group episode oven matrix silver rhythm avocado assume humble tiger shiver hurt" (DO NOT USE THIS MNEMONIC) |
| `node`      | ✓           | http://localhost:14265 | http://localhost:14265                                                                                                                                                              |
| `coin-type` | ✓           | 4219 (=Shimmer)        | 4218 (=IOTA)                                                                                                                                                                        |

#### Examples

Initialise the wallet with a randomly generated mnemonic and the default node.
```sh
./wallet init
```

Initialise the wallet with a given mnemonic and the default node.
DO NOT USE THIS MNEMONIC.
```sh
./wallet init --mnemonic "aunt middle impose faith ramp kid olive good practice motor grab ready group episode oven matrix silver rhythm avocado assume humble tiger shiver hurt"
```

Initialise the wallet with a randomly generated mnemonic and a given node.
```sh
./wallet init --node http://localhost:14265
```

Initialise the wallet with a given coin type.
See [SLIP-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md) for all registered coin types.
```sh
./wallet init --coin-type 4219
```

### `./wallet mnemonic`

Generates a new random mnemonic.

#### Example

Generate a new random mnemonic.
```sh
./wallet mnemonic
```

### `./wallet new`

Creates a new account.

The wallet needs to be initialised (`init` command).

#### Parameters

| Name    | Optional  | Default       | Example |
| ------- | --------- | ------------- | ------- |
| `alias` | ✓         | Account index | main    |

#### Examples

Create a new account with the account index as alias.
```sh
./wallet new
```

Create a new account with a provided alias.
```sh
./wallet new main
```

### `./wallet restore`

Restores accounts from a stronghold backup file.

#### Parameters

| Name    | Optional  | Example           |
| ------- | --------- | ----------------- |
| `path`  | ✘         | backup.stronghold |

#### Example

Restore accounts from a stronghold backup file.
```sh
./wallet restore backup.stronghold
```

### `./wallet set-node`

Sets the node to be used for all requests.

The new node URL is persisted to the storage and all future requests will use it.

#### Parameters

| Name  | Optional  | Example                |
| ----- | --------- | ---------------------- |
| `url` | ✘         | http://localhost:14265 |

#### Example

```sh
./wallet set-node http://localhost:14265
```

### `./wallet sync`

Synchronises all accounts.

#### Example

```sh
./wallet sync
```
