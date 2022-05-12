package org.iota.wallet.example;

import java.util.Optional;
import java.util.concurrent.TimeUnit;
import java.lang.Runnable;
import java.lang.Thread;

import org.junit.Test;

import org.iota.wallet.local.*;
import org.iota.wallet.*;

public class LibraryTest {

    static {
        try {
            NativeAPI.verifyLink();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public void testMigration() {
        try {
            Migration exampleMigration = new Migration();

            Runnable myRunnable =
                new Runnable(){
                    public void run(){
                        exampleMigration.run();
                    }
                };
            Thread thread = new Thread(myRunnable);
            thread.start();
            
            TimeUnit.SECONDS.sleep(10);

            while(!exampleMigration.finished()){
                TimeUnit.SECONDS.sleep(1);
            }

            while(exampleMigration.displayMigration() < 1){
                TimeUnit.SECONDS.sleep(10);
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @Test
    public void testTransferBuilder() {
        try {
            AddressWrapper address = AddressWrapper.parse("atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r");
            TransferOutput output = new TransferOutput(address, 10, OutputKind.SIGNATURE_LOCKED_SINGLE);
            TransferOutput[] outputs = new TransferOutput[]{
                output
            };
            Transfer t = TransferBuilder.newFromOutputs(outputs).withSkipSync().finish();
            System.out.println(t);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @Test
    public void testActorSystem() {
        try {
            String actorId = "actorId";
            System.out.println("Starting ");

            final ActorCallback callback = response -> {
                System.out.println("walletEvent " + response);
            };

            Actor.iotaInitialize(callback, actorId, "./database");

            String message = "{\"message\": \"my message\"}";
            Actor.iotaSendMessage(message);

            Actor.iotaDestroy(actorId);
            System.out.println("Done! ");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
