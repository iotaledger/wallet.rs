
  Pod::Spec.new do |s|
    s.name = 'IotaWalletMobile'
    s.version = '0.0.1'
    s.summary = 'Capacitor plugin binding to the wallet library'
    s.license = { :type => 'Apache-2.0' }
    s.homepage = 'https://github.com/iotaledger/wallet.rs#readme'
    s.author = 'IOTA Stiftung'
    s.source = { :git => 'https://github.com/iotaledger/firefly.git', :tag => s.version.to_s }
    s.source_files = ['ios/Plugin/**/*.{swift,h,m,c,cc,mm,cpp}', 'ios/*.{h,m,swift}']
    s.ios.deployment_target  = '12.0'
    s.dependency 'Capacitor'
    s.pod_target_xcconfig = { 
      'OTHER_LDFLAGS' => '-lc++',
      'ENABLE_BITCODE' => '$(ENABLE_BITCODE_$(CONFIGURATION))',
      'ENABLE_BITCODE_Release' => 'NO', 
      'ENABLE_BITCODE_Debug' => 'YES'
    }
    s.frameworks = 'WalletFramework'
    s.vendored_frameworks = 'ios/WalletFramework.xcframework'
    
    s.platform = :ios, "12.0"
    s.xcconfig = { 'SWIFT_INCLUDE_PATHS' => ['$(PODS_TARGET_SRCROOT)/ios/Plugin/Libraries', '$(PODS_TARGET_SRCROOT)/ios/WalletFramework.xcframework'] }
    s.preserve_paths = ['ios/Plugin/Libraries/module.modulemap']
    s.preserve_paths = ['ios/WalletFramework.xcframework']
  end
