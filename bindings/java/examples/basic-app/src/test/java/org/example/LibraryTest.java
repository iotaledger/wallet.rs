package org.example;

import org.junit.Test;

import org.iota.wallet.local.*;

public class LibraryTest {
    @Test
    public void testSomeLibraryMethod() {
        try {
            NativeAPI.verifyLink();
            
            Migration exampleMigration = new Migration();
            exampleMigration.run();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
