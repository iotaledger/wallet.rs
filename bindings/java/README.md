# IOTA Wallet Java Library

Get started with the official IOTA Wallet Java library.

## Requirements

minimum Java version >= 8

## Use in your Android project (Android Studio)

1. Download the `iota-wallet-1.0.0-rc.1.jar` file from the [GitHub release](https://github.com/iotaledger/wallet.rs/releases/tag/iota-wallet-java-1.0.0-rc.1-new) and add it as a library to your project.
2. Download the `iota-wallet-1.0.0-rc.1-android.zip` file from the [GitHub release](https://github.com/iotaledger/wallet.rs/releases/tag/iota-wallet-java-1.0.0-rc.1-new), unzip it and add the `jniLibs` folder with its contents to your Android Studio project as shown below:

```
project/
├──src/
   └── main/
       ├── AndroidManifest.xml
       ├── java/
       └── jniLibs/ 
           ├── arm64-v8a/           <-- ARM 64bit
           │   └── libiota-wallet.so
           │   └── libc++_shared.so
           ├── armeabi-v7a/         <-- ARM 32bit
           │   └── libiota-wallet.so
           │   └── libc++_shared.so
           │── x86/                 <-- Intel 32bit
           │  └── libiota-wallet.so
           │  └── libc++_shared.so
           └── x86_64/              <-- Intel 64bit
              └── libiota-wallet.so
              └── libc++_shared.so
```

## Use in your Java project (Linux, macOS, Windows)

Depending on your operating system, add one of the following dependencies to your `build.gradle` file:

#### linux-x86_64
```
implementation 'org.iota:iota-wallet:1.0.0-rc.1:linux-x86_64'
```

#### windows-x86_64
```
implementation 'org.iota:iota-wallet:1.0.0-rc.1:windows-x86_64'
```

#### aarch64-apple-darwin
```
implementation 'org.iota:iota-wallet:1.0.0-rc.1:aarch64-apple-darwin'
```

#### osx-x86_64
```
implementation 'org.iota:iota-wallet:1.0.0-rc.1:osx-x86_64'
```

## Use the Library

In order to use the library, you need to create a `Wallet` instance.
**Note**: Android applications must necessarily configure a suitable storage path for the wallet to avoid problems with file system permissions. You can specify a suitable storage path using the `withStoragePath()` as illustrated below:

```java
// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class CreateAccount {
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
                // Set a suitable storage path for the wallet.
                //.withStoragePath("/data/data/com.example.myapplication/")
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Create an account.
        AccountHandle a = wallet.createAccount("Alice");

        // Print the account.
        System.out.println(a);
    }
}
```

## What's Next?

Now that you are up and running, you can get acquainted with the library using
its [how-to guides](https://wiki.iota.org/shimmer/wallet.rs/how_tos/run_how_tos/) and the
repository's [code examples](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/java/examples/src).

## Instead, build everything from scratch yourself:

If you don't like to use the provided libraries and instead want to build everything yourself from scratch:

### Build for Android:

Requirements:

- minimum Java version >= 8
- Android Studio with NDK
- latest stable version of Rust
- cargo-ndk (https://github.com/bbqsrc/cargo-ndk)

1. Generate the JAR:
```
git clone https://github.com/iotaledger/wallet.rs
cd wallet.rs/bindings/java
./gradlew jarWithoutNativeLibs
```

2. You will find the built JAR in the `lib/build/libs` directory. Add it as a library to your Android project.

3. Install the Android targets you want to support:
```
 rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
```

4. Build the native library for your Android targets:
```
cd lib/native
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86 -t x86_64 -o ./jniLibs build --release
```

5. On success, you will find the built native libraries in the `jniLibs/` directory like:
```
── jniLibs/ 
    ├── arm64-v8a/           <-- ARM 64bit
    │   └── libiota-wallet.so
    ├── armeabi-v7a/         <-- ARM 32bit
    │   └── libiota-wallet.so
    │── x86/                 <-- Intel 32bit
    │  └── libiota-wallet.so
    └── x86_64/              <-- Intel 64bit
        └── libiota-wallet.so
```

6. Each folder is missing its `libc++_shared.so`. You can find them in the configured Android NDK folder like:
```
find $ANDROID_NDK_HOME -name "libc++_shared.so"
```

7. Copy the found `libc++_shared.so` files to their respective folder inside the `jniLibs` directory:
```
── jniLibs/ 
    ├── arm64-v8a/           <-- ARM 64bit
    │   └── libiota-wallet.so
    │   └── libc++_shared.so
    ├── armeabi-v7a/         <-- ARM 32bit
    │   └── libiota-wallet.so
    │   └── libc++_shared.so
    │── x86/                 <-- Intel 32bit
    │  └── libiota-wallet.so
    │  └── libc++_shared.so
    └── x86_64/              <-- Intel 64bit
        └── libiota-wallet.so
        └── libc++_shared.so
```

8. Add the `jniLibs` folder with its contents to your Android Studio project as shown below:
```
project/
├──src/
   └── main/
       ├── AndroidManifest.xml
       ├── java/
       └── jniLibs/ 
           ├── arm64-v8a/           <-- ARM 64bit
           │   └── libiota-wallet.so
           │   └── libc++_shared.so
           ├── armeabi-v7a/         <-- ARM 32bit
           │   └── libiota-wallet.so
           │   └── libc++_shared.so
           │── x86/                 <-- Intel 32bit
           │  └── libiota-wallet.so
           │  └── libc++_shared.so
           └── x86_64/              <-- Intel 64bit
              └── libiota-wallet.so
              └── libc++_shared.so
```

### Build for Linux, macOS, Windows

Please note, following instructions build the library for your host OS/architecture only.

Requirements:

- minimum Java version >= 8
- latest stable version of Rust

1. Generate the JAR:
```
git clone https://github.com/iotaledger/wallet.rs
cd wallet.rs/bindings/java
./gradlew jar
```

2. You will find the built JAR in the `lib/build/libs` directory. Add it as a library to your Java project.