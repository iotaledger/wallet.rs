// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import Foundation
import Capacitor
import Wallet

@objc(IotaWalletMobile)
public class IotaWalletMobile: CAPPlugin {
    
    class ContextResult {
        var detail = "none"
        var call: CAPPluginCall
        init(_call: CAPPluginCall) {
            self.call = _call
        }
        func notify(data: String) {
            self.call.resolve(["result": data])
            self.call.keepAlive = false
        }
    }
    
    @objc func messageHandlerNew(_ call: CAPPluginCall) {
        do {
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
            
            let error_buffer: UnsafeMutablePointer<CChar>? = nil
            let error_buffer_size = 0
            let filename = "\(path)/iota_wallet.log"
            let level_filter = "debug"
            iota_init_logger(filename.cString(using: .utf8), level_filter.cString(using: .utf8))
            let handler: OpaquePointer? = iota_initialize(jsonString.cString(using: .utf8), error_buffer, error_buffer_size)

            call.resolve(["messageHandler": Int(bitPattern: handler)])
        } catch {
            call.reject("failed to initialize messageHandlerNew")
        }
    }

    @objc func destroy(_ call: CAPPluginCall) {
        guard let handler = call.getInt("handler") else {
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
        
        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        let callback: Callback = { response, error, context  in
            guard let context = context,
                  let response = response else { return }
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
            if let error = error {
                contextResult.detail = String(cString: error)
                contextResult.notify(data: contextResult.detail)
            }
            contextResult.detail = String(cString: response)
            contextResult.notify(data: contextResult.detail)
        }

        iota_send_message(messageHandler, jsonString, callback, context)
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

        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        let callback: Callback = { response, error, context  in
            guard let context = context,
                  let response = response else { return }
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
            if let error = error {
                contextResult.detail = String(cString: error)
                contextResult.notify(data: contextResult.detail)
            }
            contextResult.detail = String(cString: response)
            contextResult.notify(data: contextResult.detail)
        }
    
        iota_listen(messageHandler, eventChar, callback, context, error_buffer, error_buffer_size)
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
        
        let contextResult = ContextResult(_call: call)
        let context = Unmanaged<ContextResult>.passRetained(contextResult).toOpaque()
        let callback: Callback = { response, error, context  in
            guard let context = context,
                  let response = response else { return }
            let contextResult = Unmanaged<ContextResult>.fromOpaque(context).takeRetainedValue()
            if let error = error {
                contextResult.detail = String(cString: error)
                contextResult.notify(data: contextResult.detail)
            }
            contextResult.detail = String(cString: response)
            contextResult.notify(data: contextResult.detail)
        }
    
        iota_listen(messageHandler, eventChar, callback, context, error_buffer, error_buffer_size)
    }
}
