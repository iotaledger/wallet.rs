// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation
import Capacitor
import Wallet

@objc(IotaWalletMobile)
public class IotaWalletMobile: CAPPlugin {
    
    // override public func load() {
    //     let fm = FileManager.default
    //     let documents = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
    //     let path = documents.appendingPathComponent("database", isDirectory: true).path
    //     if !fm.fileExists(atPath: path) {
    //         try? fm.createDirectory(atPath: path, withIntermediateDirectories: true, attributes: nil)
    //     }
        
    //     let manager_options = """
    //     {
    //         "storagePath": "\(path)",
    //         "clientOptions": {
    //             "nodes": ["https://api.testnet.shimmer.network"],
    //             "localPow": true
    //         },
    //         "coinType": 4219,
    //         "secretManager": {
    //             "Stronghold": {
    //                 "snapshotPath": "\(path)/iota_wallet.stronghold",
    //                 "password": "yourpasswordistooweak"
    //             }
    //         }
    //     }
    //     """
    //     let options = manager_options.cString(using: .utf8)
    //     let error_buffer: UnsafeMutablePointer<CChar>? = nil
    //     let error_buffer_size = 0
    //     let filename = "\(path)/iota_wallet.log"
    //     let file_name = filename.cString(using: .utf8)
    //     let level_filter = "debug"
    //     let filter = level_filter.cString(using: .utf8)
    //     iota_init_logger(file_name, filter)
    //     let handler: OpaquePointer? = iota_initialize(options, error_buffer, error_buffer_size)
    //     print(Int(bitPattern: handler))
    // }

    // private var isInitialized: Bool = false

    @objc func messageHandlerNew(_ call: CAPPluginCall) {
        do {
            // guard !isInitialized else { return }
            guard let options = call.getObject("options") else {
                return call.reject("options are required")
            }
            guard JSONSerialization.isValidJSONObject(options) else {
                return call.reject("Invalid JSON object")
            }
            let jsonData = try? JSONSerialization.data(withJSONObject: options)
            // TODO: replacing for urls slashes temporaly, make better using Codable structs with URL type?
            let jsonString = String(data: jsonData!, encoding: .utf8)!.replacingOccurrences(of: "\\", with: "")

            let fm = FileManager.default
            let documents = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            let path = documents.appendingPathComponent("database", isDirectory: true).path
            if !fm.fileExists(atPath: path) {
                try fm.createDirectory(atPath: path, withIntermediateDirectories: true, attributes: nil)
            }
            // Exclude folder from auto-backup
            var urlPath = URL(fileURLWithPath: path, isDirectory: true)
            var values = URLResourceValues()
            values.isExcludedFromBackup = true
            try urlPath.setResourceValues(values)
            
            // call.keepAlive = true

            let error_buffer: UnsafeMutablePointer<CChar>? = nil
            let error_buffer_size = 0
            let filename = "\(path)/iota_wallet.log"
            let level_filter = "debug"
            iota_init_logger(filename.cString(using: .utf8), level_filter.cString(using: .utf8))
            let handler: OpaquePointer? = iota_initialize(jsonString.cString(using: .utf8), error_buffer, error_buffer_size)

            call.resolve(["messageHandler": Int(bitPattern: handler)])
            // isInitialized = true
        } catch {
            call.reject("failed to initialize messageHandlerNew")
        }
    }

    @objc func destroy(_ call: CAPPluginCall) {
        // guard isInitialized else {
        //     call.resolve()
        //     return
        // }
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        // https://stackoverflow.com/questions/70799271/how-to-initialize-opaquepointer-in-swift
        iota_destroy(OpaquePointer(bitPattern: handler))
        // isInitialized = false
        call.resolve()
        // TODO: we need to release calls? verify if is automatically removed as are saved
    }

    @objc func sendMessage(_ call: CAPPluginCall) {
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        
        guard let message = call.getObject("message") else {
            return call.reject("message is required")
        }
        guard JSONSerialization.isValidJSONObject(message) else {
            return call.reject("Invalid JSON object")
        }
        let jsonData = try? JSONSerialization.data(withJSONObject: message)
        // TODO: replacing for urls slashes temporaly, make better using Codable structs with URL type?
        let jsonString = String(data: jsonData!, encoding: .utf8)!.replacingOccurrences(of: "\\", with: "")
        
        call.keepAlive = true
        // class Result {
        //     var detail: OpaquePointer?

        //     init(messageHandler: OpaquePointer?) {
        //         self.detail = messageHandler
        //     }
        // }
        // let object = Result(messageHandler: messageHandler)
        
        // let context = Unmanaged<Result>.passRetained(object).toOpaque()
        // let callback: Callback = { response, error, context  in
        //     guard let context = context else {
        //         return
        //     }
        //     let object = Unmanaged<Result>.fromOpaque(context).takeRetainedValue()
        //     self.notifyListeners("walletEvent", data: ["result": object.detail])
        // }
        
//        typealias cCallback = Optional<@convention(c) (
//            Optional<UnsafePointer<Int8>>,
//            Optional<UnsafePointer<Int8>>,
//            Optional<UnsafeMutableRawPointer>
//        ) -> ()>
        func cFunction(_ block: (@escaping @convention(block) (Optional<UnsafePointer<Int8>>, Optional<UnsafePointer<Int8>>, Optional<UnsafeMutableRawPointer>) -> ())) -> (Callback) {
            return unsafeBitCast(imp_implementationWithBlock(block), to: (Callback).self)
        }
        let callback: Callback = cFunction { response, error, context  in
            let data: String = String(cString: response!)
            self.notifyListeners("walletEvent", data: ["result": data])
        }
        

        iota_send_message(messageHandler, jsonString, callback, nil)
        call.resolve()
    }

    @objc func listen(_ call: CAPPluginCall) {
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        guard let eventTypes = call.getArray("eventTypes") else {
            return call.reject("eventTypes is required")
        }
        let eventChar = eventTypes.description.cString(using: .utf8)
        
        let error_buffer: UnsafeMutablePointer<CChar>? = nil
        let error_buffer_size = 0

        call.keepAlive = true

        func cFunction(_ block: (@escaping @convention(block) (Optional<UnsafePointer<Int8>>, Optional<UnsafePointer<Int8>>, Optional<UnsafeMutableRawPointer>) -> ())) -> (Callback) {
            return unsafeBitCast(imp_implementationWithBlock(block), to: (Callback).self)
        }
        let callback: Callback = cFunction { response, error, context  in
            let data: String = String(cString: response!)
            self.notifyListeners("listen", data: ["result": data])
        }
    
        iota_listen(messageHandler, eventChar, callback, nil, error_buffer, error_buffer_size)
        call.resolve()
    }

    @objc func cleanListeners(_ call: CAPPluginCall) {
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        
        guard let eventTypes = call.getArray("eventTypes") else {
            return call.reject("eventTypes is required")
        }
        let eventChar = eventTypes.description.cString(using: .utf8)
        
        let error_buffer: UnsafeMutablePointer<CChar>? = nil
        let error_buffer_size = 0
        
        call.keepAlive = true
        
        func cFunction(_ block: (@escaping @convention(block) (Optional<UnsafePointer<Int8>>, Optional<UnsafePointer<Int8>>, Optional<UnsafeMutableRawPointer>) -> ())) -> (Callback) {
            return unsafeBitCast(imp_implementationWithBlock(block), to: (Callback).self)
        }
        let callback: Callback = cFunction { response, error, context  in
            let data: String = String(cString: response!)
            self.notifyListeners("cleanListeners", data: ["result": data])
        }
    
        iota_listen(messageHandler, eventChar, callback, nil, error_buffer, error_buffer_size)
        call.resolve()
    }
}
