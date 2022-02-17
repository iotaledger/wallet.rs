import Foundation

import XCTest
@testable import IotaWallet

class SwiftAwait: XCTestCase {

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }
    
    func testAsyncAwait() async throws {
        let wallet = IotaWallet.Wallet()!
        let request = "{\"cmd\": \"CreateAccount\", \"payload\": { \"clientOptions\": { \"node\": \"https://nodes.devnet.iota.org:443\" } }, \"signerType\": { \"type\": \"Stronghold\" } }";
        
        let response = try! await wallet.sendMessage(request)
        print("\(#function) -> \(response)")
    }

}

