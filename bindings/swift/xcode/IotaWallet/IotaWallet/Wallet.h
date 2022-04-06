// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import <Foundation/Foundation.h>

@interface Wallet : NSProxy

NS_ASSUME_NONNULL_BEGIN

typedef void (^WalletHandler) (NSString * _Nullable message, NSError * _Nullable error);

- (nullable instancetype) initWithManagerOptions:(nullable NSString*) options error:(NSError**) error;
- (void) sendMessage:(NSString*) message completion:(WalletHandler) completion;
- (BOOL) listen:(NSArray<NSString*>*) event_types handler:(WalletHandler) handler error:(NSError**) error;

NS_ASSUME_NONNULL_END

@end

