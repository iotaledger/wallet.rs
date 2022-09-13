# Account Approaches

In [wallet.rs](./../welcome.md), you can use an account model
to [create an account for each user](#multi-account-approach)
or [use one account and generate multiple addresses](#single-account-approach), which you can then link to the users in
your database. The wallet library is as flexible as possible and can back up any of your use cases.

The library supports derivation for multiple accounts from a single seed. An account is simply a deterministic
identifier from which multiple addresses can be further derived.

The library also allows consumers to assign a meaningful alias to each account. Since addresses are reusable, they can
be mapped to your users in a clear and concise way.

## Multi-Account Approach

You should use the multi-account approach if you want to create an account for each individual user. You can link the
accounts to the internal user IDs as an account alias, which are distinctly separated.

## Single Account Approach

You should use the single account approach if you want to create a single account and then create an address for each
user. You will need to link the associated addresses to the internal user IDs and store who owns which address in a
database. Most exchanges are familiar with the single account approach and find it easier to use, implement, and backup.
