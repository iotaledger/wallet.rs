// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation

extension Wallet {

    convenience init() throws {
        try self.init(managerOptions: nil)
    }
    
}
