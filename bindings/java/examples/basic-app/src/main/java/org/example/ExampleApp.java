package org.example;

import java.nio.file.Paths;
import java.util.Arrays;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;
import java.nio.file.Path;

import org.iota.wallet.*;

public class ExampleApp {
    public static void main(String[] args) {
        System.out.println("Hi im main");

        try {
            new ExampleApp();
        } catch (InterruptedException | ExecutionException | TimeoutException e) {
            e.printStackTrace();
        }
    }

    public ExampleApp() throws InterruptedException, ExecutionException, TimeoutException {
        System.out.println("Hi im ExampleApp... loading!");
        NativeAPI.verifyLink();
        System.out.println("Loaded!");

        Path storageFolder = Paths.get("./my-db");

        ManagerOptions options = new ManagerOptions();
        options.setStoragePath(storageFolder.toString());

        AccountManager manager = new AccountManager(options);
        manager.setStrongholdPassword("YepThisISSecure");
        // Generate your own for peristance:
        // String mnemonic = manager.generateMnemonic();
        String mnemonic = "quick pave evoke master often ketchup rough lecture rotate improve demise envelope rent donkey peanut dizzy lesson capital confirm expire figure slide candy allow";

        // null means "generate one for me"
        manager.storeMnemonic(AccountSignerType.STRONGHOLD, mnemonic);

        ClientOptions clientOptions = new ClientOptionsBuilder()
            .withNode("https://api.lb-0.testnet.chrysalis2.com")
            .build();
        
        Account account = manager
            .createAccount(clientOptions)
            .alias("alias1")
            .initialise();
            
        System.out.println("alias: " + account.alias());
        System.out.println("balance available: " + account.balance().getAvailable());
        System.out.println("address: " + account.generateAddress().getReadable());
        
        Message[] messages = account.listMessages(5, 0, MessageType.FAILED);
        System.out.println("Messages: " + messages.length);
        for (Message m : messages) {
            if (m.payload().isPresent()){
                MessagePayload payload = m.payload().get();
                System.out.println("Message payload: " + payload.payloadType());
            }
        }

        System.out.println("acc spent addresses " + Arrays.toString(account.listSpentAddresses()));
        System.out.println("acc unspent addresses " + Arrays.toString(account.listUnspentAddresses()));

        account.transfer(
            Transfer.builder(
                account.latestAddress().address(),
                150
            ).finish()
        );
    }
}