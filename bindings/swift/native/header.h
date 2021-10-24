#import <Foundation/Foundation.h>

typedef void (*Callback)(const char *response);

#ifdef __cplusplus
extern "C" {
#endif

void iota_initialize(Callback callback, const char *actor_id, const char *storage_path);
void iota_destroy(const char *actor_id);
void iota_send_message(const char *message);
void iota_listen(const char *actor_id, const char *id, const char *event_name);

#ifdef __cplusplus
}
#endif