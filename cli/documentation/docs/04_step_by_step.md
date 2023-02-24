# Step by step examples

In these step by step examples, we present how to create a wallet and do some of the most common use cases.

It is advised to do them all at least once in the given order to understand the workflow.

## Setup

Initialise the wallet with a given node and a randomly generated mnemonic.
```sh
./wallet init --node [NODE API URL]
> ...
> INFO  Mnemonic stored successfully
```

Create a main account.
```sh
./wallet new main
> ...
> INFO  Created account "main"
> Account "main": exit
```

Create a savings account.
```sh
./wallet new savings
> ...
> INFO  Created account "savings"
> Account "savings": exit
```

## Tokens

Get some funds from the faucet to the main account.
```sh
./wallet main
> Account "main": faucet [FAUCET ENQUEUE API URL]
> ...
> Account "main": sync
> ...
> INFO  Synced: AccountBalance ...
> Account "main": exit
```

### Send a regular amount

Get an address from the savings account.
```sh
./wallet savings
> Account "savings": addresses
> INFO  Address 0: [ADDRESS]
> Account "savings": exit
```

Send a regular amount from the main account to the savings address.
```sh
./wallet main
> Account "main": send [ADDRESS] 1000000
> ...
> INFO  Transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

### Send a micro amount

Generate a new address from the savings account.
```sh
./wallet savings
> Account "savings": new-address
> ...
> INFO  Address 1: [ADDRESS]
> Account "savings": exit
```

Send a micro amount from the main account to the savings address.
```sh
./wallet main
> Account "main": send-micro [ADDRESS] 1
> ...
> INFO  Micro transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

Check the savings balance.
```sh
./wallet savings
> Account "savings": balance
> ...
> INFO  AccountBalance ...
> Account "savings": exit
```

## Native tokens

### Mint

Mint native tokens, with foundry metadata, from the main account.
```sh
./wallet main
> Account "main": mint-native-token 1000 1000 --foundry-metadata-hex 0xabcdef
> ...
> INFO  Native token minting transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

### Send

Generate a new address from the savings account.
```sh
./wallet savings
> Account "savings": new-address
> ...
> INFO  Address 2: [ADDRESS]
> Account "savings": exit
```

Send native tokens to the savings address.
```sh
./wallet main
> Account "main": sync
> ...
> INFO  Synced: AccountBalance ...TokenId([TOKEN ID])...
> Account "main": send-native-token [ADDRESS] [TOKEN ID] 100
> INFO  Native token transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

## NFTs

### Mint

Mint an NFT.
```sh
./wallet main
> Account "main": mint-nft
> ...
> INFO  NFT minting transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

### Send

Generate a new address from the savings account.
```sh
./wallet savings
> Account "savings": new-address
> ...
> INFO  Address 3: [ADDRESS]
> Account "savings": exit
```

Send the NFT to the savings address.
```sh
./wallet main
> Account "main": sync
> ...
> INFO  Synced: AccountBalance ...NftId([NFT ID])...
> Account "main": send-nft [ADDRESS] [NFT ID]
> INFO  Nft transaction sent:
> transaction id: 0x...
> Some(BlockId(0x...))
> Account "main": exit
```

## Transactions

List the transactions of the main account.
```sh
./wallet main
> Account "main": transactions
> ...
> INFO  Transaction...
> Account "main": exit
```
