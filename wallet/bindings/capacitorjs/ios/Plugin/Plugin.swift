// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation
import Capacitor
import Wallet

@objc(IotaWalletMobile)
public class IotaWalletMobile: CAPPlugin {
    
    // Handles the Swift / C pointers with a context object using Unmanaged types
    class ContextResult {
        // the `call` will send the context response back to Javascript
        let call: CAPPluginCall?
        
        init(_call: CAPPluginCall) {
            self.call = _call
        }
        
        func resolve(type: String, data: String) {
            self.call?.resolve([type: data])
        }
        // the needed `callback` to call C header functions
        let callback: Callback = { response, error, context  in
            guard let context = context,
                  let response = response else { return }
            // Convert back into `ContextResult` Swift object type
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
            if let error = error {
                contextResult.resolve(type: "error", data: String(cString: error))
                return
            }
            contextResult.resolve(type: "result", data: String(cString: response))
        }
        
        let callbackListen: Callback = { response, error, context  in
            guard let context = context,
                  let response = response else { return }
            // retain of the object awaiting for the next message.
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).retain().takeRetainedValue()
            
            if let error = error {
                contextResult.resolve(type: "error", data: String(cString: error))
                return
            }
            contextResult.resolve(type: "result", data: String(cString: response))
        }
    }

    @objc func messageHandlerNew(_ call: CAPPluginCall) {
        do {
            guard let storagePath = call.getString("storagePath"),
                  let clientOptions = call.getObject("clientOptions"),
                  let coinType = call.getInt("coinType"),
                  let secretManager = call.getObject("secretManager") else {
                return call.reject("storagePath, clientOptions, coinType, and secretManager are required")
            }
            guard JSONSerialization.isValidJSONObject(clientOptions),
                  JSONSerialization.isValidJSONObject(secretManager) else {
                return call.reject("clientOptions or secretManager is an invalid JSON object")
            }
            let ClientOptions = try? JSONSerialization.data(withJSONObject: clientOptions)
            let stringfiedClientOptions = String(data: ClientOptions!, encoding: .utf8)!.replacingOccurrences(of: "\\", with: "")
            
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
                "clientOptions":\(String(describing: stringfiedClientOptions)),
                "coinType":\(coinType),
                "secretManager":{
                    "stronghold":{
                        "snapshotPath":"\(path)/wallet.stronghold"
                }}
            }
            """
            
            let error_buffer: UnsafeMutablePointer<CChar>? = nil
            let error_buffer_size = 0
        
            // Keep the C++ handler / pointer of the messageHandler call result
            let handler: OpaquePointer? = iota_initialize(options.cString(using: .utf8), error_buffer, error_buffer_size)
            // Convert pointer to integer keeping bit pattern
            call.resolve(["messageHandler": Int(bitPattern: handler)])
        
        } catch {
            call.reject("failed to initialize messageHandlerNew")
        }
    }

    @objc func destroy(_ call: CAPPluginCall) {
        guard let handler = call.getInt("messageHandler") else {
            return call.reject("handler is required")
        }
        iota_destroy(OpaquePointer(bitPattern: handler))
        call.resolve()
    }

    @objc func sendMessage(_ call: CAPPluginCall) {
        guard let handler = call.getInt("handler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        
        // replacing for urls slashes since it's serialized on JS
        guard let message = call.getString("message")?
                .replacingOccurrences(of: "\\", with: "") else {
            return call.reject("message is required")
        }
        
        // the object to be passed as a context data on their callback
        let contextResult = ContextResult(_call: call)
        // context var to be passed on the object callback
        // where it will be converted back to object type ready to use
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        
        let error_buffer: UnsafeMutablePointer<CChar>? = nil
        let error_buffer_size = 0
        
        iota_send_message(messageHandler, message.cString(using: .utf8), contextResult.callback, context)
    }

    @objc func listen(_ call: CAPPluginCall) {
        guard let handler = call.getInt("messageHandler") else {
            return call.reject("handler is required")
        }
        let messageHandler: OpaquePointer? = OpaquePointer(bitPattern: handler)
        guard let eventTypes = call.getArray("eventTypes") else {
            return call.reject("eventTypes is required")
        }
        
        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        
        let error_buffer: UnsafeMutablePointer<CChar>? = nil
        let error_buffer_size = 0
        
        iota_listen(
            messageHandler, eventTypes.description,
            contextResult.callbackListen, context,
            error_buffer, error_buffer_size
        )
        
        call.keepAlive = true
        call.resolve()
    }

}
