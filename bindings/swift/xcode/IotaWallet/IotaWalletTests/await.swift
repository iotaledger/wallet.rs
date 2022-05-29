// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation

import XCTest
@testable import IotaWallet

class SwiftAwait: XCTestCase {
    
    struct ManagerOptions: Codable {
        var storagePath: String?
        var clientOptions: String?
        var secretManager: String?
    }

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }
    
    func testCreateAccount() async throws {
        let wallet = try! IotaWallet.Wallet()
        
        let request = "{\"cmd\": \"CreateAccount\", \"payload\": { \"clientOptions\": { \"node\": \"https://nodes.devnet.iota.org:443\" } }, \"secretManager\": { \"type\": \"Stronghold\" } }";
        
        let response = try! await wallet.sendMessage(request)
        print("\(response)")
    }
    
    func testManagerOptions() async throws {
        let secret_manager = #"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#
        let client_options = """
        {
            "nodes":[
                {
                    "url":"https://localhost",
                    "auth":null,
                    "disabled":false
                }
            ],
            "localPow":true,
            "apiTimeout":{
                "secs":20,
                "nanos":0
            }
        }
        """
        
        let manager_options = ManagerOptions(storagePath: "teststorage", clientOptions: client_options, secretManager: secret_manager)
        
        let json = try JSONEncoder().encode(manager_options)
        let manager_options_json = String(data: json, encoding: .utf8)!
        
        let _ = try! IotaWallet.Wallet(managerOptions: manager_options_json)
    }
    
    func testEvents() async throws {
        let wallet = try! IotaWallet.Wallet()
        let expectation = self.expectation(description: "TestEvents")

        try! wallet.listen([]) { (message: String?, error: Error?) in
            print("Event received: \(message!)")
            expectation.fulfill()
        }
        
        let test_event = "{\"cmd\": \"EmitTestEvent\", \"payload\": { \"TransactionProgress\": \"SyncingAccount\" } }"
        try! await wallet.sendMessage(test_event)
        
        await self.waitForExpectations(timeout: 2) { error in
            if let err = error {
                print("\(err)")
            }
        }
    }

}

