public class AccountManager {

    private String storagePath;

    private AccountManager(AccountManager.Builder builder){
        storagePath = builder.path;
    }

    public setStrongholdPassword(String password){

    }

    public storeMnemonic(SignerType type, Optional<String> mnemonic){

    }

    public createAccount(ClientOptions options){

    }
    
    public class Builder {

        private String path;

        public Builder(){

        }

        public Builder withStoragePath(String path){
            this.path = path;
        }

        public AccountManager finish(){
            return new AccountManager();
        }
    }
}
