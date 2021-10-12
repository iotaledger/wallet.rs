package org.example;

import java.util.concurrent.TimeUnit;
import java.lang.Runnable;
import java.lang.Thread;

import org.junit.Test;

import org.iota.wallet.local.*;

public class LibraryTest {
    @Test
    public void testSomeLibraryMethod() {
        try {
            NativeAPI.verifyLink();
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

            exampleMigration.displayMigration();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
