// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import <Foundation/Foundation.h>
#import <Capacitor/Capacitor.h>

CAP_PLUGIN(IotaWalletMobile, "IotaWalletMobile",
    CAP_PLUGIN_METHOD(initLogger, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(messageHandlerNew, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(sendMessage, CAPPluginReturnPromise);
    CAP_PLUGIN_METHOD(listen, CAPPluginReturnCallback);
    CAP_PLUGIN_METHOD(destroy, CAPPluginReturnPromise);
)
