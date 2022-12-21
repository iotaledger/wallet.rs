// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation
import Capacitor
import Wallet

@objc(IotaWalletMobile)
public class IotaWalletMobile: CAPPlugin {
    
    // Handle the cross-lang pointers with a context object
    class ContextResult {
        // details will hold the result string value from C++
        var detail = ""
        // the call is used to send back the context response
        var call: CAPPluginCall
        init(_call: CAPPluginCall) {
            self.call = _call
        }
        deinit {
            print("Context result object \(detail) deinit")
        }
        func resolve(data: String) {
            self.call.resolve(["result": data])
//            self.call.keepAlive = false
        }
        func listen(data: String) {
            self.call.resolve(["result": data])
//            self.call.keepAlive = false
        }
        let callback: Callback = { response, error, context  in
            guard let context = context,
            let response = response else { return }
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
            if let error = error {
                contextResult.detail = String(cString: error)
                contextResult.resolve(data: contextResult.detail)
                return
            }
            print(type(of: response), response)
            contextResult.detail = String(cString: response)
            contextResult.resolve(data: contextResult.detail)
            return
        }
    }
    
    // TODO: we really need pass this params from swift?
    private let error_buffer: UnsafeMutablePointer<CChar>? = nil
    private let error_buffer_size = 0
    
    @objc func messageHandlerNew(_ call: CAPPluginCall) {
        do {
            print("Capacitor call messageHandlerNew received", call.jsObjectRepresentation)
            guard let storagePath = call.getString("storagePath") else {
                return call.reject("storagePath are required")
            }
            
            // prepare the internal app directory path
            let fm = FileManager.default
            guard let documents = fm.urls(
                for: .applicationSupportDirectory,
                   in: .userDomainMask
            ).first else { 
                return  call.reject("can not create the path")
            }
            let path = documents.appendingPathComponent(
                storagePath,
                isDirectory: true
            ).path
            
            if !fm.fileExists(atPath: path) {
                try fm.createDirectory(
                    atPath: path,
                    withIntermediateDirectories: true,
                    attributes: nil
                )
            }
            
            // Exclude folder from auto-backup
            var urlPath = URL(fileURLWithPath: path, isDirectory: true)
            var values = URLResourceValues()
            values.isExcludedFromBackup = true
            try urlPath.setResourceValues(values)

            // we need to modify the path on the JS object
            let options = """
            {
                "storagePath":"\(path)",
                "clientOptions":{
                    "nodes":[{
                        "url":"https://api.testnet.shimmer.network",
                        "auth":{"username":"","password":""},"disabled":false
                    }]
                },
                "coinType":4219,
                "secretManager":{
                    "stronghold":{
                        "snapshotPath":"\(path)/wallet.stronghold"
                }}
            }
            """
            print(options)
            
            // TODO: implement logger as a fn
            let filename = "\(path)/iota_wallet.log"
            let level_filter = "debug"
            iota_init_logger(filename.cString(using: .utf8), level_filter.cString(using: .utf8))
            
            // Keep the C++ handler / pointer of the messageHandler call result
            let handler: OpaquePointer? = iota_initialize(options, error_buffer, error_buffer_size)
            // Convert pointer to integer keeping bit pattern
            call.resolve(["messageHandler": Int(bitPattern: handler)])
        
        } catch {
            call.reject("failed to initialize messageHandlerNew")
        }
    }

    @objc func destroy(_ call: CAPPluginCall) {
        print("Capacitor call destroy received", call.jsObjectRepresentation)
        guard let handler = call.getInt("messageHandler") else {
            return call.reject("handler is required")
        }
        iota_destroy(OpaquePointer(bitPattern: handler))
        call.resolve()
    }

    @objc func sendMessage(_ call: CAPPluginCall) {
//        print("Capacitor call sendMessage received", call.jsObjectRepresentation)
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        
        // replacing for urls slashes since it's serialized on JS
        guard let message = call.getString("message")?
                .replacingOccurrences(of: "\\", with: "") else {
            return call.reject("message is required")
        }
//        print(message)
        
        // Keep the call awaiting the result, later inside the callback
        // the passed call will send the result and disable keep alive
        call.keepAlive = true
        
        // the object to be passed as a context data on their callback
        let contextResult = ContextResult(_call: call)
        // context var to be passed on the object callback
        // where it will be converted back to object type ready to use
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        
        iota_send_message(messageHandler, message, contextResult.callback, context)
    }

    @objc func listen(_ call: CAPPluginCall) {
        print("Capacitor call listen received", call.jsObjectRepresentation)
        guard let handler = call.getInt("messageHandler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        guard let eventTypes = call.getArray("eventTypes") else {
            return call.reject("eventTypes is required")
        }
        let eventChar = eventTypes.description.cString(using: .utf8)

        call.keepAlive = true

        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
//        let callback: Callback = { response, error, context  in
//            guard let context = context,
//            let response = response else { return }
//            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
//            if let error = error {
//                contextResult.detail = String(cString: error)
//                contextResult.listen(data: contextResult.detail)
//                return
//            }
//            print(type(of: response), response)
//            contextResult.detail = String(cString: response)
//            contextResult.listen(data: contextResult.detail)
//        }
        
        let ret = iota_listen(
            messageHandler, eventChar, contextResult.callback, context,
            error_buffer, error_buffer_size
        )
        print("ret", ret)
    }

    @objc func clearListeners(_ call: CAPPluginCall) {
        print("Capacitor call clearListeners received", call.jsObjectRepresentation)
        guard let handler = call.getInt("messageHandler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        
        guard let eventTypes = call.getArray("eventTypes") else {
            return call.reject("eventTypes is required")
        }
        let eventChar = eventTypes.description.cString(using: .utf8)
        
        call.keepAlive = true
        
        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        
//        iota_clear_listeners(
//            messageHandler, eventChar, contextResult.callback, context,
//            error_buffer, error_buffer_size
//        )
    }
}
