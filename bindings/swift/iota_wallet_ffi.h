// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif


typedef struct iota_wallet_handle iota_wallet_handle_t;

typedef void (*Callback)(const char* response, const char* error, void* context);

extern iota_wallet_handle_t* iota_initialize(const char* manager_options, char* error_buffer, size_t error_buffer_size);
extern void iota_destroy(iota_wallet_handle_t*);
extern void iota_send_message(iota_wallet_handle_t* wallet_handle, const char* message, Callback callback, void* context);
extern int8_t iota_listen(iota_wallet_handle_t* wallet_handle, const char* event_types, Callback callback, void* context, char* error_buffer, size_t error_buffer_size);

#ifdef __cplusplus
}
#endif
