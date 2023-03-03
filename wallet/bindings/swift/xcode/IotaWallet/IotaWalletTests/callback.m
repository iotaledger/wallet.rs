// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import <XCTest/XCTest.h>
#import <IotaWallet/IotaWallet.h>

@interface BlockCallback : XCTestCase

@end

@implementation BlockCallback

- (void)setUp {
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
}

- (void)testBlockCallback {
    NSError* error;
    Wallet *wallet = [[Wallet alloc] initWithManagerOptions:nil error:&error];
    
    XCTAssert(wallet, @"%@", [error localizedDescription]);
    
    NSString *message = @"{\"cmd\": \"CreateAccount\", \"payload\": { \"clientOptions\": { \"node\": \"https://nodes.devnet.iota.org:443\" } }, \"coinType\": 4219, \"secretManager\": { \"type\": \"Stronghold\" } }";
    
    XCTestExpectation *expectation = [self expectationWithDescription:@"CreateAccount"];
    [wallet sendMessage:message completion:^(NSString * _Nullable message, NSError * _Nullable error) {
        XCTAssertNotNil(message, @"%@", error);
        NSLog(@"-> %@", message);
        [expectation fulfill];
    }];
    
    [self waitForExpectationsWithTimeout:2 handler:^(NSError * _Nullable error) {
        XCTAssertNil(error, "Response timed out: %@", [error localizedDescription]);
    }];
    
}

@end
