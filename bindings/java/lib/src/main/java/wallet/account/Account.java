public class Account {
    /// The account identifier.
    String id;

    /// The account's signer type.
    SignerType signerType;

    /// The account index
    int index;

    /// The account alias.
    String alias;

    /// Time of account creation.
    Date createdAt;

    /// Time the account was last synced with the Tangle.
    Optional<Date> lastSyncedAt;

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    List<Message> messages;

    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    List<Address> addresses;

    /// The client options.
    ClientOptionsclie clientOptions;

    Path storagePath;
    
    boolean skipPersistance;

    public void setAlias(String alias){

    }

    public void list_unspent_addresses(){

    }

    public void list_spent_addresses(){

    }

    public void addresses(){

    }

    public void generate_address(){

    }

    public void list_messages(5, 0, Optional<MessageType>){

    }
}
