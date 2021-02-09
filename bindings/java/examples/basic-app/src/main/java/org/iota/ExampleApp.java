package org.iota;

import wallet.*;

public class ExampleApp {
    public static void main(String[] args) {
        System.out.println("Hi im main");
        new ExampleApp();
    }

    public ExampleApp(){
        System.out.println("Hi im ExampleApp... loading!");
        NativeAPI.verifyLink();
        System.out.println("Loaded!");
    }
}