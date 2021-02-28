package org.example;

import org.junit.Test;
import static org.junit.Assert.*;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class LibraryTest {
    @Test
    public void testSomeLibraryMethod() {
        try {
            new ExampleApp();
        } catch (InterruptedException | ExecutionException | TimeoutException e) {
            e.printStackTrace();
        }
    }
}
