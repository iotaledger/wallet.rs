// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import "Wallet.h"
#import "iota_wallet_ffi.h"

#include <stddef.h>

@interface Wallet () {
    iota_wallet_handle_t* wallet_handle;
}

@end


@implementation Wallet

(BOOL) init_logger:(NSString*) file_name level_filter: (nullable NSString*) level_filter {
    return iota_init_logger(file_name.UTF8String, level_filter.UTF8String) == 0 ? YES : NO;
}

- (nullable instancetype) initWithManagerOptions:(nullable NSString*) options error:(NSError**) error {
    char errorMessage[1024] = { 0 };
    
    wallet_handle = iota_initialize(options.UTF8String, errorMessage, sizeof(errorMessage));
    if (!wallet_handle) {
        if (error) {
            NSDictionary *userInfo = @{ NSLocalizedDescriptionKey : [NSString stringWithUTF8String:errorMessage] };
            *error = [NSError errorWithDomain:@"org.iota" code:-1 userInfo:userInfo];
        }
        self = nil;
    }
    
    return self;
}

- (void) dealloc {
    iota_destroy(wallet_handle);
}

- (void) sendMessage:(NSString *) message completion: (WalletHandler) completion {
    iota_send_message(wallet_handle, message.UTF8String, callback, (void*)CFBridgingRetain(completion));
}

- (BOOL) listen:(NSArray<NSString*>*) event_types handler: (WalletHandler) handler error:(NSError**) error {
    NSData* data = [NSJSONSerialization dataWithJSONObject:event_types options:NSJSONWritingPrettyPrinted error:nil];
    NSString* message = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
    char errorMessage[1024] = { 0 };
    
    int8_t ret = iota_listen(wallet_handle, message.UTF8String, callback, (void*)CFBridgingRetain(handler), errorMessage, sizeof(errorMessage));
    
    if (ret && error) {
        NSDictionary *userInfo = @{ NSLocalizedDescriptionKey : [NSString stringWithUTF8String:errorMessage] };
        *error = [NSError errorWithDomain:@"org.iota" code:-1 userInfo:userInfo];;
    }
    return ret == 0 ? YES : NO;
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
