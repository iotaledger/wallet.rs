package wallet.account;

import java.nio.file.Path;
import java.util.Date;
import java.util.List;

import wallet.types.Address;
import wallet.types.SignerType;
import wallet.types.ClientOptions;
import wallet.types.Message;

public class AccountInitialiser {
    private AccountStore accounts;
    private Path storagePath;
    private String alias;
    private Date createdAt;
    private List<Message> messages;
    private List<Address> addresses;
    private ClientOptions clientOptions;
    private SignerType signerType;
    private boolean skipPersistance;

    public AccountInitialiser(ClientOptions client_options, AccountStore accounts, Path storagePath){
        
    }

    public AccountInitialiser signerType(SignerType signerType){
        this.signerType = signerType;
        return this;
    }

    /// Defines the account alias. If not defined, we'll generate one.
    public AccountInitialiser alias(String alias){
        this.alias = alias;
        return this;
    }

    /// Time of account creation.
    public AccountInitialiser created_at(Date createdAt){
        this.createdAt = createdAt;
        return this;
    }

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    public AccountInitialiser messages(List<Message> messages){
        this.messages = messages;
        return this;
    }

    // Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    public AccountInitialiser addresses(List<Address> addresses){
        this.addresses = addresses;
        return this;
    }

    /// Skips storing the account to the database.
    public AccountInitialiser skipPersistance(){
        this.skipPersistance = true;
        return this;
    }

    public Account initialise(){
        return new Account();
    }
}

