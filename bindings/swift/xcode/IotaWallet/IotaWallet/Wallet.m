// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import "Wallet.h"
#import "iota_wallet_ffi.h"

@interface Wallet () {
    iota_wallet_handle_t* wallet_handle;
}

- (nullable instancetype) initWalletHandle:(const char *) path;

@end


@implementation Wallet

- (nullable instancetype) initWalletHandle:(const char *) path {
    wallet_handle = iota_initialize(NULL);
    if (!wallet_handle) {
        self = nil;
    }
    return self;
}

- (nullable instancetype) init {
    return [self initWalletHandle: NULL];
}

- (nullable instancetype) initWithStoragePath:(NSString*) path {
    return [self initWalletHandle: path.UTF8String];
}

- (void) dealloc {
    iota_destroy(wallet_handle);
}

- (void) sendMessage:(NSString *) message completion: (WalletHandler) completion {
    iota_send_message(wallet_handle, message.UTF8String, callback, (void*)CFBridgingRetain(completion));
}

static void callback(const char* response, const char* error, void* context)
{
    WalletHandler handler = CFBridgingRelease(context);
    NSError *returnError = nil;
    NSString *message = nil;
    
    if (error) {
        NSDictionary *userInfo = @{ NSLocalizedDescriptionKey : [NSString stringWithUTF8String:error] };
        returnError = [NSError errorWithDomain:@"org.iota" code:-1 userInfo:userInfo];
    } else if (response) {
        message = [NSString stringWithUTF8String:response];
    }
    
    
    handler(message, returnError);
}

@end
