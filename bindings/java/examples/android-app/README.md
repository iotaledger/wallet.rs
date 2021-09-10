Requirements:
- Android NDK
- Clang toolchain

Android Toolchains:
```
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

set `ANDROID_NDK_HOME` environment variable
Example: `export ANDROID_NDK_HOME=$ANDROID_HOME/ndk-bundle`
If you dont have `ANDROID_HOME`; Usually found at `/home/user/Android`

install cargo-ndk: `cargo install cargo-ndk`

Then run gradle:
./gradlew aR

Have a signing keystore ready; I call it `signed_apk.jks`

Sign the apk:
$ANDROID_HOME/build-tools/28.0.3/apksigner sign --ks examples/android-app/signed_apk.jks --out examples/android-app/android-app-release-signed.apk -v examples/android-app/build/outputs/apk/release/android-app-release-unsigned.apk

Connect device:
`adb pair 192.168.0.x:x` 
`adb connect 192.168.0.x:x`

Run on device:
`adb install -r --fastdeploy examples/android-app/android-app-release-signed.apk`

Monitor app start:
`adb shell am monitor`