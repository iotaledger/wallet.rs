package org.example;

import java.nio.file.Paths;
import java.util.Arrays;
import java.nio.file.Path;

import org.iota.wallet.*;
import org.iota.wallet.local.*;

public class ExampleApp implements ErrorListener, StrongholdStatusListener {
    public static void main(String[] args) {

        try {
            new ExampleApp();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public ExampleApp() {
        NativeAPI.verifyLink();

        EventManager.subscribeErrors(this);
        EventManager.subscribeStrongholdStatusChange(this);

        Path storageFolder = Paths.get("./my-db");

        // Beware: All builder patterns return NEW instances on each method call.
        // Mutating the old builder after a builder call will not result in a change on
        // the second call
        // This is due to the JNI bindings not beeing able to call non-reference methods
        // in rust
        // Examble that doesnt work:
        // AccountManagerBuilder builder = AccountManager.Builder();
        // builder.withStorage(storageFolder.toString(), null);
        // AccountManager manager = builder.finish();
        //
        // Explanation: builder.withStorage returns a new builder instance, and .finish
        // is called on the old one
        AccountManagerBuilder builder = AccountManager.Builder().withStorage(storageFolder.toString(), null);

        AccountManager manager = builder.finish();
        manager.setStrongholdPassword("YepThisISSecure");

        // Generate your own for peristance:
        // String mnemonic = manager.generateMnemonic();
        
        // null means "generate one for me"
        manager.storeMnemonic(AccountSignerType.STRONGHOLD, null);

        BrokerOptions mqtt = new BrokerOptions();
        
        ClientOptions clientOptions = new ClientOptionsBuilder()
            .withNode("https://api.lb-0.testnet.chrysalis2.com")
            .withMqttBrokerOptions(mqtt)
            .build();
        
        Account account = manager
            .createAccount(clientOptions)
            .signerType(AccountSignerType.STRONGHOLD)
            .alias("alias1")
            .initialise();

        System.out.println("id: " + account.id());
        System.out.println("alias: " + account.alias());
        System.out.println("balance available: " + account.balance().getAvailable());
        System.out.println("address: " + account.generateAddress().getReadable());
        
        Message[] messages = account.listMessages(5, 0, MessageType.FAILED);
        System.out.println("Messages: " + messages.length);
        for (Message m : messages) {
            if (m.payload().isPresent()){
                printMessage(m.payload().get());
            }
        }

        System.out.println("acc spent addresses " + Arrays.toString(account.listSpentAddresses()));
        System.out.println("acc unspent addresses " + Arrays.toString(account.listUnspentAddresses()));

        try {
            account.transfer(Transfer.builder(account.latestAddress().address(), 150).finish());
        } catch (WalletException e) {
            System.out.println(e.getMessage());
        }
    }

    private void printMessage(MessagePayload payload){
        System.out.println("Message payload type: " + payload.payloadType());
        System.out.println("Message payload: " + payload);
    }

    private void hexTest(){
        String test = "Yes im a test!";
        String hex = RustHex.encode(test);
        
        System.out.println(Arrays.toString(test.getBytes()));
        System.out.println(hex);
        System.out.println(Arrays.toString(RustHex.decode(hex)));
    }

    @Override
    public void onError(String error) {
        System.out.println("ON_ERROR: " + error);
    }

    @Override
    public void onStrongholdStatusChange(StrongholdStatusEvent event) {
        System.out.println("STRONGHOLD STATUS: " + event.status());
        System.out.println("seconds left: " + event.unlockedDuration().getSeconds());
    }
}
