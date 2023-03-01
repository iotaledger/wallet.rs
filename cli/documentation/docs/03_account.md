# Account Interface

The Account Interface is evaluated, after the Account Manager Interface, repeatedly through a prompt within the `wallet`
binary.

It is responsible for the creation and management of account addresses and their outputs, tokens, native tokens, NFTs...

## Commands

### `addresses`

Lists all account addresses.

#### Example

```sh
> Account "main": addresses
```

### `balance`

Prints the account balance.

#### Example

```sh
> Account "main": balance
```

### `burn-native-token`

Burns a native token.

#### Parameters

| Name       | Optional  | Default | Example                                                                        |
| ---------- | --------- | ------- | ------------------------------------------------------------------------------ |
| `token_id` | ✘         | N/A     | 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 |
| `amount`   | ✘         | N/A     | 100                                                                            |

#### Example

Burn the provided amount of a native token.
```sh
> Account "main": burn-native-token 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 100
```

### `burn-nft`

Burns an NFT.

#### Parameters

| Name     | Optional  | Default | Example                                                            |
| -------- | --------- | ------- | ------------------------------------------------------------------ |
| `nft_id` | ✘         | N/A     | 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c |

#### Example

Burn a provided NFT.
```sh
> Account "main": burn-nft 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c
```

### `claim`

Tries to claim outputs with storage deposit return, expiration or timelock unlock conditions.

#### Parameters

| Name        | Optional  | Default | Example                                                                |
| ----------- | --------- | ------- | ---------------------------------------------------------------------- |
| `output_id` | ✓         | N/A     | 0xd5dff9ee869dfa7796d5132b220cb5c00146c36abba27d3562e2d726decb50850000 |

#### Examples

Try to claim all outputs with storage deposit return, expiration or timelock unlock conditions.
```sh
> Account "main": claim
```

Try to claim a specific output.
```sh
> Account "main": claim 0xd5dff9ee869dfa7796d5132b220cb5c00146c36abba27d3562e2d726decb50850000
```

### `clear`

Clears the terminal.

#### Example

```sh
> Account "main": clear
```

### `consolidate`

Tries to consolidate outputs into a single one.

Note that only Basic Outputs with only an address unlock condition can be consolidated.

#### Example

```sh
> Account "main": consolidate
```

### `create-alias-output`

Create a new alias output.

#### Example

```sh
> Account "main": create-alias-output
```

### `decrease-native-token-supply`

Melts a native token.

#### Parameters

| Name       | Optional  | Default | Example                                                                        |
| ---------- | --------- | ------- | ------------------------------------------------------------------------------ |
| `token_id` | ✘         | N/A     | 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 |
| `amount`   | ✘         | N/A     | 100                                                                            |

#### Example

Melt the provided amount of a native token.
```sh
> Account "main": decrease-native-token-supply 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 100
```

### `decrease-voting-power`

Decreases the voting power of the account.

#### Parameters

| Name       | Optional  | Default | Example  |
| ---------- | --------- | ------- | -------- |
| `amount`   | ✘         | N/A     | 100000   |

#### Example

Decrease the voting power of the account by 100000.
```sh
> Account "main": decrease-voting-power 100000
```

### `destroy-alias`

Destroys an alias.

#### Parameters

| Name       | Optional  | Default | Example                                                            |
| ---------- | --------- | ------- | ------------------------------------------------------------------ |
| `alias_id` | ✘         | N/A     | 0xb2bcba092bfb3fe3a12afcf21115016b27d833a7c456404fe2fe0921799f24dd |

#### Example

```sh
> Account "main": destroy-alias 0xb2bcba092bfb3fe3a12afcf21115016b27d833a7c456404fe2fe0921799f24dd
```

### `destroy-foundry`

Destroys a foundry.

#### Parameters

| Name         | Optional  | Default | Example                                                                        |
| ------------ | --------- | ------- | ------------------------------------------------------------------------------ |
| `foundry_id` | ✘         | N/A     | 0x08b2bcba092bfb3fe3a12afcf21115016b27d833a7c456404fe2fe0921799f24dd0100000000 |

#### Example

```sh
> Account "main": destroy-foundry 0x08b2bcba092bfb3fe3a12afcf21115016b27d833a7c456404fe2fe0921799f24dd0100000000
```

### `exit`

Exits the `cli-wallet`.

#### Example

```sh
> Account "main": exit
```

### `faucet`

Requests funds from a faucet.

#### Parameters

| Name      | Optional  | Default                           | Example                                                         |
| --------- | --------- | --------------------------------- | --------------------------------------------------------------- |
| `url`     | ✓         | http://localhost:8091/api/enqueue | http://localhost:8091/api/enqueue                               |
| `address` | ✓         | The latest address of the account | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |

#### Examples

Request funds from a given faucet to the latest account address.
```sh
> Account "main": faucet http://localhost:8091/api/enqueue
```

Request funds from a given faucet to a given address.
```sh
> Account "main": faucet http://localhost:8091/api/enqueue rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3
```

### `help`

Displays the account interface usage.

#### Example

```sh
> Account "main": help
```

### `increase-native-token-supply`

Mints more of a native token.

#### Parameters

| Name       | Optional  | Default | Example                                                                        |
| ---------- | --------- | ------- | ------------------------------------------------------------------------------ |
| `token_id` | ✘         | N/A     | 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 |
| `amount`   | ✘         | N/A     | 10                                                                             |

#### Example

Mint 10 additional native tokens.
```sh
> Account "main": increase-native-token-supply 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 10
```

### `increase-voting-power`

Increases the voting power of the account.

#### Parameters

| Name       | Optional  | Default | Example  |
| ---------- | --------- | ------- | -------- |
| `amount`   | ✘         | N/A     | 100000   |

#### Example

Increase the voting power of the account by 100000.
```sh
> Account "main": increase-voting-power 100000
```

### `mint-native-token`

Mints a native token.

#### Parameters

| Name                      | Optional  | Default | Example       |
| ------------------------- | --------- | ------- | ------------- |
| `circulating_supply`      | ✘         | N/A     | 1000          |
| `maximum_supply`          | ✘         | N/A     | 1000          |
| `foundry_metadata_hex`    | ✓         | None    | 0xabcdef      |
| `foundry_metadata_file`   | ✓         | None    | metadata.json |

#### Examples

Mint a native token with a maximum supply.
```sh
> Account "main": mint-native-token 1000 1000
```

Mint a native token with a maximum supply and hexadecimal foundry metadata.
```sh
> Account "main": mint-native-token 1000 1000 --foundry-metadata-hex 0xabcdef
```

Mint a native token with a maximum supply and foundry metadata from a file.
```sh
> Account "main": mint-native-token 1000 1000 --foundry-metadata-file metadata.json
```

### `mint-nft`

Mints an NFT.

[IOTA NFT Standard - TIP27](https://github.com/iotaledger/tips/blob/main/tips/TIP-0027/tip-0027.md).

#### Parameters

| Name                      | Optional  | Default                           | Example                                                         |
| ------------------------- | --------- | --------------------------------- | --------------------------------------------------------------- |
| `address`                 | ✓         | The first address of the account  | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |
| `immutable_metadata_hex`  | ✓         | None                              | 0xabcdef                                                        |
| `immutable_metadata_file` | ✓         | None                              | metadata.json                                                   |
| `metadata_hex`            | ✓         | None                              | 0xabcdef                                                        |
| `metadata_file`           | ✓         | None                              | metadata.json                                                   |
| `tag`                     | ✓         | None                              | 0xabcdef                                                        |
| `sender`                  | ✓         | None                              | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |
| `issuer`                  | ✓         | None                              | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |

#### Examples

Mint an NFT to the latest address of the account.
```sh
> Account "main": mint-nft
```

Mint an NFT to a given address.
```sh
> Account "main": mint-nft rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3
```

Mint an NFT to a given address with hexadecimal immutable metadata and metadata from a file.
```sh
> Account "main": mint-nft rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 --immutable-metadata-hex 0xabcdef --metadata-file metadata.json
```

Mint an NFT to a given address with hexadecimal tag and sender feature.
```sh
> Account "main": mint-nft --tag 0xabcdef --sender rms1qq5k0ut6nl2vpyehdvg5k4ygyntd4r44t9lw2ksex280x60lc2fmcgdsmku
```

### `new-address`

Generates a new address.

#### Example

```sh
> Account "main": new-address
```

### `output`

Displays an output that is stored in the account.

#### Parameters

| Name        | Optional  | Default | Example                                                                |
| ----------- | --------- | ------- | ---------------------------------------------------------------------- |
| `output_id` | ✘         | N/A     | 0x1c7a765db0c1f5eceb0ea5578585359c5b0c1ab8d958829f5990997b93f0ec7d0100 |

#### Example

```sh
> Account "main": output 0x1c7a765db0c1f5eceb0ea5578585359c5b0c1ab8d958829f5990997b93f0ec7d0100
```

### `outputs`

Displays all outputs that are stored in the account.

#### Example

```sh
> Account "main": outputs
```

### `participation-overview`

Calculates the participation overview of the account.

#### Parameters

| Name       | Optional  | Default | Example                                                            |
| ---------- | --------- | ------- | ------------------------------------------------------------------ |
| `event_id` | ✓         | N/A     | 0x8d6ffcd8d6c9f049b5732adb8900bb6f6a28e282b15a2297405a28181eebd515 |

#### Example

Get the participation overview for all events.
```sh
> Account "main": participation-overview
```

Get the participation overview only for a specific event.
```sh
> Account "main": participation-overview 0x8d6ffcd8d6c9f049b5732adb8900bb6f6a28e282b15a2297405a28181eebd515
```

### `send`

Sends an amount to an address.

#### Parameters

| Name      | Optional  | Default | Example                                                         |
| --------- | --------- | ------- | --------------------------------------------------------------- |
| `address` | ✘         | N/A     | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |
| `amount`  | ✘         | N/A     | 1000000                                                         |

#### Example

```sh
> Account "main": send rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 1000000
```

### `send-micro`

Sends a micro amount to an address with StorageDepositReturn and Expiration Unlock Conditions.

#### Parameters

| Name      | Optional  | Default | Example                                                         |
| --------- | --------- | ------- | --------------------------------------------------------------- |
| `address` | ✘         | N/A     | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 |
| `amount`  | ✘         | N/A     | 1                                                               |

#### Example

```sh
> Account "main": send-micro rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 1
```

### `send-native-token`

Sends native tokens to an address with StorageDepositReturn and Expiration Unlock Condition.

To send the native tokens together with the required storage deposit and without StorageDepositReturn and Expiration, provide `true` for `gift_storage_deposit`.

#### Parameters

| Name                   | Optional  | Default | Example                                                                         |
| ---------------------- | --------- | ------- | ------------------------------------------------------------------------------- |
| `address`              | ✘         | N/A     | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3                 |
| `token_id`             | ✘         | N/A     | 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000  |
| `amount`               | ✘         | N/A     | 100                                                                             |
| `gift_storage_deposit` | ✓         | false   | true                                                                            |

#### Example

Sending with storage deposit return and expiration:

```sh
> Account "main": send-native-token rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 100
```

Sending without storage deposit return and expiration, gifting the required storage deposit:

```sh
> Account "main": send-native-token rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 0x08860e1f3593ba86c597cf86f61d8b04d8a714c02c7c5da7132d45be9c2ce6445c0300000000 100 true
```

### `send-nft`

Sends an NFT to an address.

#### Parameters

| Name      | Optional  | Default | Example                                                             |
| --------- | --------- | ------- | ------------------------------------------------------------------- |
| `address` | ✘         | N/A     | rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3     |
| `nft_id`  | ✘         | N/A     | 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c  |

#### Example

```sh
> Account "main": send-nft rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c
```

### `stop-participating`

Stops participating to a given event.

#### Parameters

| Name          | Optional  | Default | Example                                                                |
| ------------- | --------- | ------- | ---------------------------------------------------------------------- |
| `event_id`    | ✘         | N/A     | `0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c`   |

#### Example

```sh
> Account "main": stop-participating 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c
```

### `sync`

Synchronises the account.

#### Example

```sh
> Account "main": sync
```

### `transactions`

Lists all account transactions.

#### Example

```sh
> Account "main": transactions
```

### `unspent-outputs`

Displays all unspent outputs that are stored in the account.

#### Example

```sh
> Account "main": unspent-outputs
```

### `vote`

Casts given votes for a given event.

#### Parameters

| Name          | Optional         | Default | Example                                                                 |
| ------------- | ---------------- | ------- | ----------------------------------------------------------------------- |
| `event_id`    | ✘                | N/A     | `0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c`    |
| `answers`     | ✘ (at least one) | N/A     | 0 1 1 0                                                                 |

#### Example

```sh
> Account "main": vote 0x397ae8552dcf0dc604a44c9d86a5005d09f95d67e2965ea3b1c1271f9a9ae44c 0 1 1 0
```

### `voting-output`

Gets the voting output of the account.

#### Example

```sh
> Account "main": voting-output
```

### `voting-power`

Gets the voting power of the account.

#### Example

```sh
> Account "main": voting-power
```