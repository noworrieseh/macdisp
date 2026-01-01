#ifndef DISPLAY_SERVICES_H
#define DISPLAY_SERVICES_H

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint32_t width;
    uint32_t height;
    double refresh_rate;
    uint32_t depth;
    uint32_t mode_number;
    bool is_stretched;
    bool is_interlaced;
    bool is_tv_mode;
    bool is_safe_for_hardware;
    bool is_scaled;  // HiDPI/Retina scaling
} DisplayMode;

typedef struct {
    DisplayMode *modes;
    size_t count;
} DisplayModeList;

// Check if DisplayServices framework is available
bool ds_is_available(void);

// Get all available modes for a display (using DisplayServices if available)
DisplayModeList *ds_get_all_modes(uint32_t display_id);

// Get current mode for a display
DisplayMode *ds_get_current_mode(uint32_t display_id);

// Set display mode (returns 0 on success)
int ds_set_mode(uint32_t display_id, uint32_t mode_number);

// Free mode list
void ds_free_mode_list(DisplayModeList *list);

// Free single mode
void ds_free_mode(DisplayMode *mode);

#endif // DISPLAY_SERVICES_H
