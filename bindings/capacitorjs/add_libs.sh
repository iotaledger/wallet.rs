#!/bin/sh

set -e
rm -rf tmp && mkdir tmp && cd tmp

curl -SL --progress-bar --fail https://github.com/iotaledger/wallet.rs/releases/download/iota-wallet-java-1.0.0-rc.1-new/iota-wallet-1.0.0-rc.1-android.zip > iota-wallet.zip
unzip iota-wallet.zip             

rm -rf ../android/src/main/jniLibs
cp -r jniLibs ../android/src/main/jniLibs

curl -SL --progress-bar --fail https://github.com/iotaledger/wallet.rs/releases/download/iota-wallet-java-1.0.0-rc.1-new/iota-wallet-1.0.0-rc.1.jar > iota-wallet.jar
rm -rf ../android/libs && mkdir -p ../android/libs             
cp -r iota-wallet.jar ../android/libs

cd .. && rm -rf tmp
echo "success!"