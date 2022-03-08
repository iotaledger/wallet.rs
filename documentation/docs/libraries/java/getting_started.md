---
description: Getting started with the official IOTA Wallet Library Java binding.
image: /img/logo/iota_mark_light.png
keywords:
- Java
- Rust
- jar
- maven
- environment variable
---
# Getting Started with Java

## Prerequisite

To use the library, we recommend you update Rust to the latest stable version [`$ rustup update stable`](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). Nightly should be fine but some changes might not be compatible.

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md). Then also install the following programs:

- Java & JDK (Make sure $JAVA_HOME env variable) is set
- [Gradle](https://gradle.org/install/) v4 or higher or [Maven](https://maven.apache.org/download.cgi) installe
- Cargo ndk (`cargo install cargo-ndk`)
- Cargo fmt (`rustup component add rustfmt`)


Download or clone the `wallet.rs` repository
```
$ git clone https://github.com/iotaledger/iota.rs.git
```

## Security

:::warning
In a production setup, do not store passwords in the host's environment variables or in the source code.  See our [backup and security recommendations](https://chrysalis.docs.iota.org/guides/backup_security) for production setups.
:::

## Installation

In order to build using the Wallet.rs Java bindings, you need the following two parts:
1. JNI Native library linking `Rust` to `C`, and then `C` to java `native` methods (`.so` , `.dll` or `.dylib` depending on your system)
2. Java archive(Jar) containing `native` methods which call C code. (`.jar`)

### Step 1: Creating the native library

Build the wallet library (This generates the java source code and JNI library file)
```
cd wallet.rs/bindings/java
cargo build --release
```

Generated binaries can then be found at `wallet.rs/bindings/java/target/release`

:::info
Compiling for Android requires additional compilation instructions.
Specific instructions can be found in [Android development](android_development.md)
:::

### Step 2: Creating the Java archive

#### Generating the source files and classes

After step 1, Java source files will be generated under `wallet.rs/bindings/java/native/src/main/java/org/iota/wallet`.

If this step succeeded, we need to generate the jar file containing the newly generated Java source files.

#### Generating the jar

Generating the jar can be done with your tool of preference. We provide examples for Gradle and Maven in this guide

##### Gradle

Make `gradlew` executable (`chmod +x gradlew`) if needed, then run
```
cd wallet.rs/bindings/java
./gradlew jar
```

##### Maven
```
cd wallet.rs/bindings/java
mvn install
```

After running one of these commands, the jar can then be found at `wallet.rs/bindings/java/native/build/libs/native.jar`


## Usage
You can find more information on using the `wallet.rs` library's java binding in the [examples section](examples.md).

### Gradle
```
./gradlew examples:java-app:test --info
```

### Maven
```
mvn exec:exec
```

## API Reference

API Reference can be found [here](api_reference).

## Limitations

Due to the fact that we are linking through C from Rust, there are a couple of limiting factors.

- Classic builder patterns return a `clone` after each builder call since we can only pass back to C by reference in `Rust`
```Java
Builder builder1 = new Builder();
Builder builder2 = builder1.setValue(true);

// These are different instances, thus builder1 wont have the value set
assertNotEquals(builder1, builder2);
```
