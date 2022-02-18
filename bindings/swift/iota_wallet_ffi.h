// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#ifdef __cplusplus
extern "C" {
#endif

typedef struct iota_wallet_handle iota_wallet_handle_t;

typedef void (*Callback)(const char* response, const char* error, void* context);

extern iota_wallet_handle_t* iota_initialize(const char* storage_path);
extern void iota_destroy(iota_wallet_handle_t*);
extern void iota_send_message(iota_wallet_handle_t* wallet_handle, const char* message, Callback callback, void* context);
// extern void iota_listen(const char* actor_id, const char* id, const char* event_name);


#ifdef __cplusplus
}
#endif
