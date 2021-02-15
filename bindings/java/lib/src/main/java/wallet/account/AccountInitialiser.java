public class AccountInitialiser {
    accounts: AccountStore,
    storage_path: PathBuf,
    alias: Option<String>,
    created_at: Option<DateTime<Local>>,
    messages: Vec<Message>,
    addresses: Vec<Address>,
    client_options: ClientOptions,
    signer_type: Option<SignerType>,
    skip_persistance: bool

    pub fn signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type.replace(signer_type);
        self
    }

    /// Defines the account alias. If not defined, we'll generate one.
    pub fn alias(mut self, alias: impl AsRef<str>) -> Self {
        self.alias = Some(alias.as_ref().to_string());
        self
    }

    /// Time of account creation.
    pub fn created_at(mut self, created_at: DateTime<Local>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    // Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    pub fn addresses(mut self, addresses: Vec<Address>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Skips storing the account to the database.
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    public Account initialise(){
        
    }
}

