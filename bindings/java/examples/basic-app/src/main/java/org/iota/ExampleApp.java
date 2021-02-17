package org.iota;

import java.nio.file.Paths;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.nio.file.Path;

import wallet.NativeAPI;
import wallet.types.*;
import wallet.accountmanager.*;
import wallet.account.*;

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
        AccountManager manager = 
            new AccountManager.Builder()
                .withStoragePath(storageFolder)
                .finish();

        ClientOptions clientOptions = new ClientOptions.Builder()
            .withNode("http://api.lb-0.testnet.chrysalis2.com")
            .build();

        Account account = manager
            .createAccount(clientOptions);

        System.out.println("alias " + account.alias());
        System.out.println("balance " + account.balance());
        
        System.out.println("syncing account now");
    
        boolean synced = account.sync().get(500, TimeUnit.MILLISECONDS);
    
        System.out.println("synced " + synced);
        System.out.println("acc messages " + account.listMessages(5, 0, MessageType.FAILED));
        System.out.println("acc spent addresses " + account.listSpentAddresses());
        System.out.println("acc unspent addresses " + account.listUnspentAddresses());
    }
}