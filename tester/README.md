The wallet tester is a test engine that allows running tests that are described in JSON format so that no knowledge of Rust is required to write tests.

Each test file is composed of three sections that are executed sequentially.

# Accounts

Describes the initial state of the wallet.
It is an array of accounts where each item creates a new account with the same index.

An item can be:
- A value, which creates an account with a single `BasicOutput` with the given amount; 

    ```json
        "accounts": [
            50000,
            0
        ],
    ```

    Creates
    + Account `0` with one `BasicOutput` of amount `50000`:
    + Account `1` with no output;


- An array, which creates 

    ```json
        "accounts": [
            [
                50000,
                75000
            ],
            0
        ],
    ```

    Creates:
    + Account `0` with one `BasicOutput` of amount `50000` and another one of amount `75000`;
    + Account `1` with no output;

# Steps

Describes the different steps to be executed on the wallet.

# Checks

Describes the checks to be performed on the final state of the wallet.