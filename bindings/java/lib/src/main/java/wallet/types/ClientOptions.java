package wallet.types;

public class ClientOptions {

    private String node;

    private ClientOptions(ClientOptions.Builder builder){
        node = builder.node;
    }
    
    public static class Builder {

        private String node;

        public Builder(){

        }

        public Builder withNode(String node){
            this.node = node;
            return this;
        }

        public ClientOptions build(){
            return new ClientOptions(this);
        }
    }
}
