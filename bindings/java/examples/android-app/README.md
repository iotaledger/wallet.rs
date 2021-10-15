# Requirements:
For this setup we use `$ANDROID_NDK_HOME` for the location of your NDK, wether you use Android studio or manual compilation

- Android NDK or Android Studio with NDK installed (If you extract make sure to make it executable "chmod -R +x android-ndk-XYZ" )
- Clang toolchain
- Cargo ndk (`cargo install cargo-ndk`)

Android Toolchains: 
```
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

# Setup

### Adding shared library
For each target you enable in `build.gradle` `archTriplets` do the following:
> Copy `$ANDROID_NDK_HOME/sources/cxx-stl/llvm-libc++/libs/ARCH/libc++_shared.so`
> to `src/main/libs/ARCH/`

### Cross compile note
Currently cross compiling has only worked on WSL/Linux.
If you wish to use android studio in wondows, first make the binaries in WSL/Linux, then copy them over to `src/main/jniLibs/ARCH/`. Afterwards you need to comment out all `archTriplets` in `build.gradle`. You will still need to copy the `libc++_shared.so` from the step above for each ARCH


## Android studio

Load the project under the `wallet.rs/bindings/java` folder in Android studio.

Make sure you have an NDK and SDK: `file->Project Structure->SDK Location`. If the NDK location is marked grey, edit the local.properties like so: (This must be the location of `$ANDROID_NDK_HOME`)
```
ndk.dir=I\:\\Path\\To\\AndroidSDK\\ndk\\VERSION
```

If youre on linux/wsl, just run the app. On other platforms see the `Setup/Cross compile note` before running.

## Manual

set `ANDROID_NDK_HOME` environment variable
Example: `export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/VERSION`
If you dont have `ANDROID_HOME`; Usually found at `/home/user/Android`

Make sure you copied the shared libraries from the `Setup/Adding shared library` step.

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