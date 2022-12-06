// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import <Foundation/Foundation.h>
#import <Capacitor/Capacitor.h>

CAP_PLUGIN(IotaWalletMobile, "IotaWalletMobile",
    CAP_PLUGIN_METHOD(sendMessage, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(destroy, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(initialize, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(listen, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(cleanListeners, CAPPluginReturnPromise);
)
