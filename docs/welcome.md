# Welcome
This is the documentation for the official IOTA Wallet Library Software. It can be used to easily integrate an IOTA Wallet into your applications. You can read more about core principles behind IOTA client libraries in the following blog [post](https://blog.iota.org/the-new-iota-client-libraries-harder-better-faster-stronger/).

`Wallet.rs` is a general wallet library written in Rust. It is being utilized by our wallet software `Firefly` and other software components across IOTA ecosystem. `Wallet.rs` contains all the logic to safely build wallets or integrations that require value-based transfers (such as exchanges, pay-as-you-go systems, etc.). It includes account state management and backup, account creation, transferring tokens and much more. Needless to say, it is also based on our official `one-source-code-of-truth` [IOTA Rust library](https://github.com/iotaledger/iota.rs) and can be integrated with our [Stronghold enclave](https://blog.iota.org/iota-stronghold-6ce55d311d7c/) to achieve a maximum level of security.

> Using `stronghold` is a recommended approach to store account data using `wallet.rs`. One get the best security practices for free 

With the `wallet.rs` library, developers do not need to use a self-generated seed anymore. By default, the seed is created and stored in Stronghold encrypted at rest. It is not possible to extract the seed from Stronghold for security purposes. Stronghold uses encrypted snapshots that can easily be backed up and securely shared between devices. These snapshots are further secured with a password.

## IOTA 1.5 (Chrysalis) in a nutshell
* IOTA network uses a DAG (Directed Acyclic Graph) to store its transactions. Each transaction can reference up to 8 parent transactions
* There is a breaking change moving from IOTA 1.0 to IOTA 1.5 (Chrysalis). IOTA address was originally based on WOTS signature scheme (81 trytes) and it has been replaced by a Ed25519 signature scheme (Bech32 [checksummed base32 format] string of 64 characters)
* In contrast to IOTA 1.0, IOTA 1.5 addresses are perfectly resuable: so even if one spent funds from the given address it can be used again
* There are new client libraries developed in rust, specifically `iota.rs`, `wallet.rs` and `stronghold.rs` that serve as `one-source-code-of-truth` to IOTA users and providing binding to other programming languages 
* Example of new format of the IOTA 1.5 address (Bech32 string):
<table>
    <thead>
        <tr>
            <th colspan=4><center>iota11qykf7rrdjzhgynfkw6z7360avhaaywf5a4vtyvvk6a06gcv5y7sksu7n5cs</center></th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan=4><center>three distinguished parts</center></td>
        </tr>
        <tr>
            <td><center><strong>human-readable</strong></center></td>
            <td><center><strong>separator</strong></center></td>
            <td><center><strong>data</strong></center></td>
            <td><center><strong>checksum</strong></center></td>
        </tr>
        <tr>
            <td><center>iota | atoi</center></td>
            <td><center>1</center></td>
            <td><center>48 bytes [0..9a..z]</center></td>
            <td><center>6 characters [0..9a..z]</center></td>
        </tr>
        <tr>
            <td><center>iota</center></td>
            <td><center>1</center></td>
            <td><center>1qykf7rrdjzhgynfkw6z7360avhaaywf5a4vtyvvk6a06gcv5y7sks</center></td>
            <td><center>u7n5cs</center></td>
        </tr>
        <tr>
            <td colspan=4>iota = mainnet; atoi = testnet</td>
        </tr>
    </tbody>
</table>

## Warning
This library is in active development. The library targets the Chrysalis testnet and does not work with current IOTA mainnet.

## Testnet
To join the Chrysalis public testnet checkout [this link](https://blog.iota.org/chrysalis-phase-2-testnet-out-now/). More information about Chrysalis components is available at [documentation portal](https://chrysalis.docs.iota.org/).

## Joining the discussion
If you want to get involved in discussions about this library, or you're looking for support, go to the #clients-discussion channel on [Discord](https://discord.iota.org).

## What you will find here
This documentation has five paths:
1. The Overview, an detailed overview of the wallet library. 
2. Libraries, all avaiable programming languages and their resources.
3. The Specification, detailed explaination requirements and functionality.
4. Contribute, how you can work on the wallet software.
5. Get in touch, join the community and become part of the X-Team!
