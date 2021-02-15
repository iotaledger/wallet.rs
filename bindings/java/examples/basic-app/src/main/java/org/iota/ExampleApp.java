package org.iota;

import wallet.*;

public class ExampleApp {
    public static void main(String[] args) {
        System.out.println("Hi im main");
        new ExampleApp();
    }

    public ExampleApp(){
         /**
        let storage_folder: PathBuf = "./my-db".into();
        let manager =
            AccountManager::builder()
                .with_storage_path(&storage_folder)
                .finish()
                .await?;
        let client_options = ClientOptionsBuilder::new().with_node("http://api.lb-0.testnet.chrysalis2.com")?.build();
        let account = manager
            .create_account(client_options)?
            .initialise()
            .await?;
          */

        System.out.println("Hi im ExampleApp... loading!");
        NativeAPI.verifyLink();
        System.out.println("Loaded!");
    }
}