package wallet.accountmanager;

import java.nio.file.Path;
import java.util.Optional;

import wallet.types.*;
import wallet.account.Account;

public class AccountManager {

    private Path storagePath;

    private AccountManager(AccountManager.Builder builder){
        storagePath = builder.path;
    }

    public void setStrongholdPassword(String password){

    }

    public void storeMnemonic(SignerType type, Optional<String> mnemonic){

    }

    public Account createAccount(ClientOptions options){
        return new Account();
    }
    
    public static class Builder {

        private Path path;

        public Builder(){

        }

        public Builder withStoragePath(Path path){
            this.path = path;
            return this;
        }

        public AccountManager finish(){
            return new AccountManager(this);
        }
    }
}
