import json

class LedgerNanoSecretManager:
    def __init__(self, is_simulator):
        """Initialize a ledger nano secret manager.
        """

        self.LedgerNano = is_simulator
    def toJSON(self):
        return json.dumps(self, default=lambda o: o.__dict__)
            
class MnemonicSecretManager:
    def __init__(self, mnemonic):
        """Initialize a mnemonic secret manager.
        """

        self.Mnemonic = mnemonic
    def toJSON(self):
        return json.dumps(self, default=lambda o: o.__dict__)

class StrongholdSecretManager:
    def __init__(self, snapshot_path, password):
        """Initialize a stronghold secret manager.
        """

        self.Stronghold = StrongholdSecretManager.Inner(snapshot_path, password)
    def toJSON(self):
        return json.dumps(self, default=lambda o: o.__dict__)

    class Inner:
        def __init__(self, snapshot_path, password):
            self.password = password
            self.snapshotPath = snapshot_path