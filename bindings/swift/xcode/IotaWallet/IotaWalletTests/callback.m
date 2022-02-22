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
    Wallet *wallet = [[Wallet alloc] init];
    
    NSString *message = @"{\"cmd\": \"CreateAccount\", \"payload\": { \"clientOptions\": { \"node\": \"https://nodes.devnet.iota.org:443\" } }, \"signerType\": { \"type\": \"Stronghold\" } }";
    
    XCTestExpectation *expectation = [self expectationWithDescription:@"CreateAccount"];
    [wallet sendMessage:message completion:^(NSString * _Nullable message, NSError * _Nullable error) {
        NSLog(@"-> %@", message);
        [expectation fulfill];
    }];
    
    [self waitForExpectationsWithTimeout:2 handler:^(NSError * _Nullable error) {
        if (error) {
            XCTAssert(false, "Response timed out: %@", [error localizedDescription]);
        }
    }];
    
}

@end
