#!/bin/sh
set -e

FRAMEWORK_NAME=IOTAWalletInternal

./make_framework.sh --target aarch64-apple-darwin
./make_framework.sh --target aarch64-apple-ios
./make_framework.sh --target x86_64-apple-ios

mkdir -p target/universal/
rm -rf target/universal/$FRAMEWORK_NAME.xcframework
mkdir target/universal/$FRAMEWORK_NAME.xcframework

mkdir -p target/aarch64-apple-darwin/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/macos-arm64
cp -r target/aarch64-apple-darwin/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/macos-arm64/

mkdir -p target/aarch64-apple-ios/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/ios-arm64
cp -r target/aarch64-apple-ios/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/ios-arm64/

mkdir -p target/x86_64-apple-ios/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/ios-x86
cp -r target/x86_64-apple-ios/$FRAMEWORK_NAME.framework target/universal/$FRAMEWORK_NAME.xcframework/ios-x86/

touch target/universal/$FRAMEWORK_NAME.xcframework/Info.plist
tee -a target/universal/$FRAMEWORK_NAME.xcframework/Info.plist > /dev/null <<EOT
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AvailableLibraries</key>
	<array>
		<dict>
			<key>LibraryIdentifier</key>
			<string>macos-arm64</string>
			<key>LibraryPath</key>
			<string>${FRAMEWORK_NAME}.framework</string>
			<key>SupportedArchitectures</key>
			<array>
				<string>arm64</string>
			</array>
			<key>SupportedPlatform</key>
			<string>macos</string>
		</dict>
		<dict>
			<key>LibraryIdentifier</key>
			<string>ios-arm64</string>
			<key>LibraryPath</key>
			<string>${FRAMEWORK_NAME}.framework</string>
			<key>SupportedArchitectures</key>
			<array>
				<string>arm64</string>
			</array>
			<key>SupportedPlatform</key>
			<string>ios</string>
		</dict>
		<dict>
			<key>LibraryIdentifier</key>
			<string>ios-x86</string>
			<key>LibraryPath</key>
			<string>${FRAMEWORK_NAME}.framework</string>
			<key>SupportedArchitectures</key>
			<array>
				<string>x86_64</string>
			</array>
			<key>SupportedPlatform</key>
			<string>ios</string>
			<key>SupportedPlatformVariant</key>
			<string>simulator</string>
		</dict>
	</array>
	<key>CFBundlePackageType</key>
	<string>XFWK</string>
	<key>XCFrameworkFormatVersion</key>
	<string>1.0</string>
</dict>
</plist>
EOT
echo "Exported in target/universal/${FRAMEWORK_NAME}.xcframework"
