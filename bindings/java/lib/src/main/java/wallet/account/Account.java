package wallet.account;

import java.nio.file.Path;
import java.util.Date;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;

import wallet.types.*;
import wallet.account.Account;

public class Account {
    /// The account identifier.
    private String id;

    /// The account's signer type.
    private SignerType signerType;

    /// The account index
    private int index;

    /// The account alias.
    private String alias;

    /// Time of account creation.
    private Date createdAt;

    /// Time the account was last synced with the Tangle.
    private Optional<Date> lastSyncedAt;

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    private List<Message> messages;

    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    private List<Address> addresses;

    /// The client options.
    private ClientOptions clientOptions;

    private Path storagePath;
    
    private boolean skipPersistance;

    private ExecutorService executor 
      = Executors.newSingleThreadExecutor();

    public void setAlias(String alias){

    }

    public String alias(){
        return this.alias;
    }

    public int balance(){
        return 1;
    }

    public String listUnspentAddresses(){
        return "";
    }

    public String listSpentAddresses(){
        return "";
    }

    public void addresses(){

    }

    public void generateAddress(){

    }

    public String listMessages(int a, int b, MessageType type){
        return "";
    }
    
    public Future<Boolean> sync(){
        return executor.submit(() -> {
            Thread.sleep(1000);
            return true;
        });
    }
}
