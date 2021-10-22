# Dependencies:
For this setup we use `$ANDROID_NDK_HOME` for the location of your NDK, wether you use Android studio or manual compilation

- Dependencies of the Wallet.rs README.md for compiling wallet.rs normally
- Java & JDK (Make sure JAVA_HOME env variable) is set
- Android NDK or Android Studio with NDK installed (If you extract make sure to make it executable `chmod -R +x android-ndk-VERSION` )
- [Rustup](https://rustup.rs/)
- Cargo ndk (`cargo install cargo-ndk`)
- Cargo fmt (`rustup component add rustfmt`)

Android Toolchains: 
```
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

# Setup

### Generating the java files
In order to generate the Java files; we need to run manually cargo once. 
This step will require `cargo build --release --target=$TARGET` in `wallet.rs/bindings/java/native`.
Replace `$TARGET` with one of the enabled targets inside you `build.gradle` `archTriplets` (options are armeabi-v7a, arm64-v8a, x86, x86_64)

### Cross compile note

In order to build on windows, we need to add android triplets to our VCPKG. 
[TODO]

Currently cross compiling has only worked on WSL/Linux.
If you wish to use android studio in windows, first make the android target binaries in WSL/Linux, then copy them over to `src/main/jniLibs/$TARGET/`. Afterwards you need to comment out all `archTriplets` in `build.gradle`. You will still need to copy the `libc++_shared.so` from the step above for each ARCH

## Android studio

Load the project under the `wallet.rs/bindings/java` folder in Android studio.

Make sure you have an NDK and SDK: `file->Project Structure->SDK Location`. If the NDK location is marked grey, edit the local.properties like so: (This must be the location of `$ANDROID_NDK_HOME`, which still needs to be on path)
```
ndk.dir=I\:\\Path\\To\\AndroidSDK\\ndk\\VERSION
```

If youre on linux/wsl, just run the app. On other platforms see the `Setup/Cross compile note` before running.

## Manual

set `ANDROID_NDK_HOME` environment variable
Example: `export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/VERSION`
If you dont have `ANDROID_HOME`; Usually found at `/home/user/Android`

### Adding shared library
For each target you enable in `build.gradle` `archTriplets` do the following:
> Copy `$ANDROID_NDK_HOME/sources/cxx-stl/llvm-libc++/libs/$TARGET/libc++_shared.so`
> to `src/main/libs/ARCH/`

`$TARGET` Should be replaced with each enabled `archTriplets` key. (options are armeabi-v7a, arm64-v8a, x86, x86_64)

Then run gradle:
./gradlew aR

Have a signing keystore ready; I call it `signed_apk.jks`
How to make: https://developer.android.com/studio/publish/app-signing#generate-key

Sign the apk:
$ANDROID_HOME/build-tools/VERSION/apksigner sign --ks examples/android-app/signed_apk.jks --out examples/android-app/android-app-release-signed.apk -v examples/android-app/build/outputs/apk/release/android-app-release-unsigned.apk

Connect device:
`adb pair 192.168.0.x:x` 
`adb connect 192.168.0.x:x`

Run on device:
`adb install -r --fastdeploy examples/android-app/android-app-release-signed.apk`

Monitor app start:
`adb shell am monitor`