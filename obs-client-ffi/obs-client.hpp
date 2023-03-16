#ifndef OBS_CLIENT_H
#define OBS_CLIENT_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>
typedef void *Capture;

struct Frame {
    uintptr_t width;
    uintptr_t height;
    uint8_t *data;
};

extern "C" {

Capture *create_capture(const char *name_str);

void free_capture(Capture *capture);

bool try_launch_capture(Capture *capture);

Frame *capture_frame(Capture *capture);

} // extern "C"

#endif // OBS_CLIENT_H
