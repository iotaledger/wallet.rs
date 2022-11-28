# Guild to Run the Python Examples

## Build the wheel file

- Go to `wallet.rs/bindings/python`
- `python3 setup.py bdist_wheel`

## Create a virtual environment and use it (optional)

- `python3 -m venv .venv`
- `source .venv/bin/activate`

## Install the wheel file

- Go to `wallet.rs/bindings/python/dist`
- `python3 -m pip install [your built wheel file]`

## Run examples

- Go to `wallet.rs/bindings/python/examples`
- `python3 [example file]`

## To deactivate the virtual environment (optional)

- `deactivate`