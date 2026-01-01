#import <Foundation/Foundation.h>
#import <CoreGraphics/CoreGraphics.h>
#import <IOKit/graphics/IOGraphicsLib.h>
#import "display_services.h"
#import <dlfcn.h>
#import <string.h>

// Mode description buffer size
#define MODE_DESC_SIZE 256

// CGS API function pointer types for Sequoia
typedef int (*CGSGetNumberOfDisplayModes_t)(uint32_t displayID, int *outCount);
typedef int (*CGSGetDisplayModeDescription_t)(uint32_t displayID, int idx, int *outData);
typedef int (*CGSGetCurrentDisplayMode_t)(uint32_t displayID, int *outModeNum);
typedef int (*CGSConfigureDisplayMode_t)(void *config, uint32_t displayID, int modeNum);

// Global function pointers
static void *ds_handle = NULL;
static CGSGetNumberOfDisplayModes_t cgs_get_num_modes = NULL;
static CGSGetDisplayModeDescription_t cgs_get_mode_desc = NULL;
static CGSGetCurrentDisplayMode_t cgs_get_current = NULL;
static CGSConfigureDisplayMode_t cgs_configure = NULL;
static bool ds_initialized = false;

static DisplayMode mode_from_cgs(int *buffer) {
    DisplayMode mode = {0};

    // Based on test output:
    // buffer[0] = mode_id
    // buffer[1] = flags (various display mode properties)
    // buffer[2] = width
    // buffer[3] = height
    // buffer[4] = depth
    // buffer[9] = refresh_rate

    mode.mode_number = buffer[0];
    mode.width = buffer[2];
    mode.height = buffer[3];
    mode.depth = buffer[4];
    mode.refresh_rate = buffer[9];
    mode.is_safe_for_hardware = true;

    // HiDPI/scaled detection:
    // Modes with width <= 1800 are marked as "scaling:on" in displayplacer
    // These represent true 2x HiDPI modes where 1 logical pixel = 2 physical pixels
    // Larger resolutions (1920+) are "looks like" modes without the scaling flag
    mode.is_scaled = (mode.width <= 1800 && (buffer[1] & 0x0F) != 0);

    return mode;
}

static DisplayMode mode_from_cg(CGDisplayModeRef mode_ref) {
    DisplayMode mode = {0};

    mode.width = (uint32_t)CGDisplayModeGetWidth(mode_ref);
    mode.height = (uint32_t)CGDisplayModeGetHeight(mode_ref);
    mode.refresh_rate = CGDisplayModeGetRefreshRate(mode_ref);
    mode.depth = 32;  // Default since CGDisplayModeCopyPixelEncoding is deprecated
    mode.mode_number = (uint32_t)CGDisplayModeGetIODisplayModeID(mode_ref);
    mode.is_safe_for_hardware = true;

    return mode;
}

static void ds_init(void) {
    if (ds_initialized) return;
    ds_initialized = true;

    // Load DisplayServices framework (it's in the dyld shared cache on Sequoia)
    ds_handle = dlopen("/System/Library/PrivateFrameworks/DisplayServices.framework/DisplayServices", RTLD_LAZY);

    if (!ds_handle) {
        return;
    }

    // Load CGS functions (Sequoia API)
    cgs_get_num_modes = dlsym(ds_handle, "CGSGetNumberOfDisplayModes");
    cgs_get_mode_desc = dlsym(ds_handle, "CGSGetDisplayModeDescription");
    cgs_get_current = dlsym(ds_handle, "CGSGetCurrentDisplayMode");
    cgs_configure = dlsym(ds_handle, "CGSConfigureDisplayMode");

    // If we didn't find the functions, close the handle
    if (!cgs_get_num_modes || !cgs_get_mode_desc || !cgs_get_current) {
        dlclose(ds_handle);
        ds_handle = NULL;
    }
}

bool ds_is_available(void) {
    ds_init();
    return ds_handle != NULL && cgs_get_num_modes != NULL;
}

DisplayModeList *ds_get_all_modes(uint32_t display_id) {
    ds_init();

    DisplayModeList *list = malloc(sizeof(DisplayModeList));
    if (!list) return NULL;

    if (ds_is_available()) {
        // Use CGS API (Sequoia)
        int count = 0;
        if (cgs_get_num_modes(display_id, &count) != 0 || count <= 0) {
            free(list);
            return NULL;
        }

        list->count = (size_t)count;
        list->modes = malloc(sizeof(DisplayMode) * count);
        if (!list->modes) {
            free(list);
            return NULL;
        }

        for (int i = 0; i < count; i++) {
            int buffer[MODE_DESC_SIZE] = {0};
            if (cgs_get_mode_desc(display_id, i, buffer) == 0) {
                list->modes[i] = mode_from_cgs(buffer);
            }
        }
    } else {
        // Fallback to CoreGraphics
        CFArrayRef modes_array = CGDisplayCopyAllDisplayModes(display_id, NULL);
        if (!modes_array) {
            free(list);
            return NULL;
        }

        CFIndex count = CFArrayGetCount(modes_array);
        list->count = (size_t)count;
        list->modes = malloc(sizeof(DisplayMode) * count);

        for (CFIndex i = 0; i < count; i++) {
            CGDisplayModeRef mode_ref = (CGDisplayModeRef)CFArrayGetValueAtIndex(modes_array, i);
            list->modes[i] = mode_from_cg(mode_ref);
        }

        CFRelease(modes_array);
    }

    return list;
}

DisplayMode *ds_get_current_mode(uint32_t display_id) {
    ds_init();

    DisplayMode *mode = malloc(sizeof(DisplayMode));
    if (!mode) return NULL;

    if (ds_is_available()) {
        // Use CGS API
        int mode_num = 0;
        if (cgs_get_current(display_id, &mode_num) != 0) {
            free(mode);
            return NULL;
        }

        // The mode_num IS the correct index - use it directly
        int buffer[MODE_DESC_SIZE] = {0};
        if (cgs_get_mode_desc(display_id, mode_num, buffer) == 0) {
            *mode = mode_from_cgs(buffer);
        } else {
            // Fallback: just set the mode number
            mode->mode_number = mode_num;
        }
    } else {
        // Fallback to CoreGraphics
        CGDisplayModeRef mode_ref = CGDisplayCopyDisplayMode(display_id);
        if (!mode_ref) {
            free(mode);
            return NULL;
        }

        *mode = mode_from_cg(mode_ref);
        CGDisplayModeRelease(mode_ref);
    }

    return mode;
}

int ds_set_mode(uint32_t display_id, uint32_t mode_number) {
    ds_init();

    // If DisplayServices is available, use CGSConfigureDisplayMode for HiDPI/scaled modes
    if (ds_is_available() && cgs_configure) {
        CGDisplayConfigRef config;
        CGBeginDisplayConfiguration(&config);

        int result = cgs_configure(config, display_id, (int)mode_number);

        if (result == 0) {
            CGError error = CGCompleteDisplayConfiguration(config, kCGConfigureForSession);
            return (error == kCGErrorSuccess) ? 0 : -1;
        } else {
            CGCancelDisplayConfiguration(config);
            // Fall through to CoreGraphics method
        }
    }

    // Fallback to CoreGraphics for native modes
    CFArrayRef modes_array = CGDisplayCopyAllDisplayModes(display_id, NULL);
    if (!modes_array) return -1;

    CGDisplayModeRef target_mode = NULL;
    CFIndex count = CFArrayGetCount(modes_array);

    // Try to find mode by resolution/refresh rate match
    if (ds_is_available()) {
        int buffer[MODE_DESC_SIZE] = {0};
        if (cgs_get_mode_desc(display_id, mode_number, buffer) == 0) {
            int target_width = buffer[2];
            int target_height = buffer[3];
            int target_hz = buffer[9];

            for (CFIndex i = 0; i < count; i++) {
                CGDisplayModeRef mode = (CGDisplayModeRef)CFArrayGetValueAtIndex(modes_array, i);
                int mode_width = (int)CGDisplayModeGetWidth(mode);
                int mode_height = (int)CGDisplayModeGetHeight(mode);
                int mode_hz = (int)CGDisplayModeGetRefreshRate(mode);

                if (mode_width == target_width &&
                    mode_height == target_height &&
                    mode_hz == target_hz) {
                    target_mode = mode;
                    break;
                }
            }
        }
    } else {
        // Match by mode ID
        for (CFIndex i = 0; i < count; i++) {
            CGDisplayModeRef mode = (CGDisplayModeRef)CFArrayGetValueAtIndex(modes_array, i);
            if ((uint32_t)CGDisplayModeGetIODisplayModeID(mode) == mode_number) {
                target_mode = mode;
                break;
            }
        }
    }

    int result = -1;
    if (target_mode) {
        CGDisplayConfigRef config;
        CGBeginDisplayConfiguration(&config);
        CGError error = CGConfigureDisplayWithDisplayMode(config, display_id, target_mode, NULL);
        if (error == kCGErrorSuccess) {
            error = CGCompleteDisplayConfiguration(config, kCGConfigureForSession);
            result = (error == kCGErrorSuccess) ? 0 : -1;
        } else {
            CGCancelDisplayConfiguration(config);
        }
    }

    CFRelease(modes_array);
    return result;
}

void ds_free_mode_list(DisplayModeList *list) {
    if (list) {
        if (list->modes) {
            free(list->modes);
        }
        free(list);
    }
}

void ds_free_mode(DisplayMode *mode) {
    if (mode) {
        free(mode);
    }
}

char *ds_get_display_uuid(uint32_t display_id) {
    // CGDisplayCreateUUIDFromDisplayID is not available in public API
    // Use IOKit to get the UUID instead
    io_service_t service = CGDisplayIOServicePort(display_id);
    if (!service) {
        // Fallback: create a pseudo-UUID from display ID
        char *buffer = malloc(64);
        if (buffer) {
            snprintf(buffer, 64, "%08X-0000-0000-0000-000000000000", display_id);
        }
        return buffer;
    }

    CFDictionaryRef info = IODisplayCreateInfoDictionary(service, kIODisplayOnlyPreferredName);
    if (!info) {
        char *buffer = malloc(64);
        if (buffer) {
            snprintf(buffer, 64, "%08X-0000-0000-0000-000000000000", display_id);
        }
        return buffer;
    }

    // Try to get display properties for UUID generation
    CFNumberRef vendorID = CFDictionaryGetValue(info, CFSTR(kDisplayVendorID));
    CFNumberRef productID = CFDictionaryGetValue(info, CFSTR(kDisplayProductID));
    CFNumberRef serialNum = CFDictionaryGetValue(info, CFSTR(kDisplaySerialNumber));

    uint32_t vendor = 0, product = 0, serial = 0;
    if (vendorID) CFNumberGetValue(vendorID, kCFNumberSInt32Type, &vendor);
    if (productID) CFNumberGetValue(productID, kCFNumberSInt32Type, &product);
    if (serialNum) CFNumberGetValue(serialNum, kCFNumberSInt32Type, &serial);

    char *buffer = malloc(64);
    if (buffer) {
        snprintf(buffer, 64, "%08X-%04X-%04X-%04X-%08X%04X",
                 vendor, product, (serial >> 16) & 0xFFFF, serial & 0xFFFF,
                 display_id, (vendor ^ product) & 0xFFFF);
    }

    CFRelease(info);
    return buffer;
}

char *ds_get_display_type(uint32_t display_id) {
    const char *type_str;

    if (CGDisplayIsBuiltin(display_id)) {
        type_str = "MacBook built in screen";
    } else {
        type_str = "External display";
    }

    return strdup(type_str);
}

int ds_configure_display(uint32_t display_id, int x, int y, int rotation,
                         uint32_t mirror_display_id, bool enabled) {
    CGDisplayConfigRef config;
    CGError error = CGBeginDisplayConfiguration(&config);

    if (error != kCGErrorSuccess) {
        return -1;
    }

    // Set origin (position)
    if (x != -1 && y != -1) {
        error = CGConfigureDisplayOrigin(config, display_id, x, y);
        if (error != kCGErrorSuccess) {
            CGCancelDisplayConfiguration(config);
            return -1;
        }
    }

    // Set rotation - not available in public API
    // Rotation requires using private APIs or would need to be done through DisplayServices
    if (rotation != -1 && rotation != 0) {
        // Log that rotation is not supported in this implementation
        fprintf(stderr, "Warning: Display rotation is not supported via public APIs\n");
    }

    // Set mirroring
    if (mirror_display_id != 0) {
        error = CGConfigureDisplayMirrorOfDisplay(config, display_id, mirror_display_id);
        if (error != kCGErrorSuccess) {
            CGCancelDisplayConfiguration(config);
            return -1;
        }
    }

    // Enable/disable display - not directly supported in public API
    // The main display cannot be disabled
    if (!enabled) {
        fprintf(stderr, "Warning: Disabling displays is not supported via public APIs\n");
    }

    error = CGCompleteDisplayConfiguration(config, kCGConfigureForSession);
    return (error == kCGErrorSuccess) ? 0 : -1;
}

void ds_free_string(char *str) {
    if (str) {
        free(str);
    }
}
