#ifndef OBS_CLIENT_H
#define OBS_CLIENT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef void *Capture;

typedef struct Frame {
    uintptr_t width;
    uintptr_t height;
    uint8_t *data;
} Frame;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

Capture *create_capture(const char *name_str);

void free_capture(Capture *capture);

bool try_launch_capture(Capture *capture);

struct Frame *capture_frame(Capture *capture);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* OBS_CLIENT_H */
