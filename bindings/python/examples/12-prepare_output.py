from iota_wallet import IotaWallet

# In this example we will prepare an output with an address and expiration unlock condition and send it

wallet = IotaWallet("./alice-database")

account = wallet.get_account("Alice")

wallet.set_stronghold_password("some_hopefully_secure_password")

# using prepare_output
output = account.prepare_output(
    {
        "amount": "1000000",
        "recipientAddress": "rms1qprutadk4uc9rp6h7wh7sech42sl0z40ztpgyygr5tf0cn5jrqshgm8y43d",
        "unlocks":
            {
                "expirationUnixTime": 1676570528,
            },
    }
)
print(f"Output: {output}")

# using build_basic_output
output = account.build_basic_output(
    amount="1000000",
    native_tokens=[],
    unlock_conditions=[
        {
            "type": 0,
            "address": {
                "type": 0,
                "pubKeyHash": "0x47c5f5b6af30518757f3afe86717aaa1f78aaf12c2821103a2d2fc4e92182174",
            },
        },
        {
            "type": 3,
            "returnAddress": {
                "type": 0,
                "pubKeyHash": "0x8297ac4149c80cca8225e5f2da36df89a93cd2998d7f6d488c97250a881e65af",
            },
            "unixTime": 1676570528,
        },
    ],
    features=[],
)
print(f"Output: {output}")

account.sync()

transaction = account.send_outputs([output])
print(f'Sent transaction: {transaction}')
