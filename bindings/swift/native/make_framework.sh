#!/bin/sh
set -e

while [ $# -gt 0 ]; do
	if [[ $1 == *"--"* ]]; then
        v="${1/--/}"
        declare $v="$2"
	fi
	shift
done

if [ -z "$target" ]; then 
	echo "--target is missing"
	exit 1
fi

FRAMEWORK_NAME=IOTAWalletInternal
FRAMEWORK_VERSION=1.0.0

makeFramework() {
	cd $1
	rm -rf $FRAMEWORK_NAME.framework
	mkdir $FRAMEWORK_NAME.framework
	mkdir $FRAMEWORK_NAME.framework/Headers
	mkdir $FRAMEWORK_NAME.framework/Modules

	touch $FRAMEWORK_NAME.framework/Modules/Modules.modulemap
	tee -a $FRAMEWORK_NAME.framework/Modules/Modules.modulemap > /dev/null <<EOT
framework module $FRAMEWORK_NAME {
  umbrella header "$FRAMEWORK_NAME.h"

  export *
  module * { export * }
}
EOT

	touch $FRAMEWORK_NAME.framework/Info.plist
	tee -a $FRAMEWORK_NAME.framework/Info.plist > /dev/null <<EOT
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
   <key>CFBundleDevelopmentRegion</key>
   <string>en</string>
   <key>CFBundleExecutable</key>
   <string>${FRAMEWORK_NAME}</string>
   <key>CFBundleIdentifier</key>
   <string>org.iotafoundation.${FRAMEWORK_NAME}</string>
   <key>CFBundleInfoDictionaryVersion</key>
   <string>6.0</string>
   <key>CFBundleName</key>
   <string>${FRAMEWORK_NAME}</string>
   <key>CFBundlePackageType</key>
   <string>FMWK</string>
   <key>CFBundleShortVersionString</key>
   <string>1.0</string>
   <key>CFBundleVersion</key>
   <string>${FRAMEWORK_VERSION}</string>
</dict>
</plist>
EOT

	cp ../../header.h $FRAMEWORK_NAME.framework/Headers/$FRAMEWORK_NAME.h
	cp release/libwallet.a $FRAMEWORK_NAME.framework/$FRAMEWORK_NAME
	cd ../../
}
rustup target add ${target}
cargo build --target ${target} --release --lib
#cargo lipo --release --targets ${target}
makeFramework target/${target}
echo "Exported in target/${target}/${FRAMEWORK_NAME}.framework"
