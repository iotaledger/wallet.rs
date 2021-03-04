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
        // null means "generate one for me"
        manager.storeMnemonic(AccountSignerType.STRONGHOLD, null);

        ClientOptions clientOptions = new ClientOptionsBuilder()
            .with_node("https://api.lb-0.testnet.chrysalis2.com")
            .build();
        
        Account account = manager
            .create_account(clientOptions)
            .alias("alias1")
            .initialise();
            
        System.out.println("alias: " + account.alias());
        System.out.println("balance available: " + account.balance().getAvailable());
        System.out.println("address: " + account.generate_address().getReadable());
        
        System.out.println("acc messages " + Arrays.toString(account.list_messages(5, 0, MessageType.FAILED)));
        System.out.println("acc spent addresses " + Arrays.toString(account.list_spent_addresses()));
        System.out.println("acc unspent addresses " + Arrays.toString(account.list_unspent_addresses()));

        account.transfer(
            Transfer.builder(
                account.latest_address().address(),
                150
            ).finish()
        );
    }
}