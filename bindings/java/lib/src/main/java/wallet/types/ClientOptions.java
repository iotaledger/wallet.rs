public class ClientOptions {

    private String node;

    private ClientOptions(ClientOptions.Builder builder){
        node = builder.node;
    }
    
    public class Builder {

        private String node;

        public Builder(){

        }

        public Builder withnode(String node){
            this.node = node;
        }

        public ClientOptions build(){
            return new ClientOptions();
        }
    }
}
