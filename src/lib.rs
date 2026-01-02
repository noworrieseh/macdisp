use core_graphics::display::{CGDisplayBounds, CGGetActiveDisplayList};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f64,
    pub depth: u32,
    pub mode_number: u32,
    pub is_stretched: bool,
    pub is_interlaced: bool,
    pub is_tv_mode: bool,
    pub is_safe_for_hardware: bool,
    pub is_scaled: bool,
}

#[repr(C)]
struct DisplayModeList {
    modes: *mut DisplayMode,
    count: usize,
}

extern "C" {
    fn ds_is_available() -> bool;
    fn ds_get_display_uuid(display_id: u32) -> *mut std::os::raw::c_char;
    fn ds_get_display_type(display_id: u32) -> *mut std::os::raw::c_char;
    fn ds_get_all_modes(display_id: u32) -> *mut DisplayModeList;
    fn ds_get_current_mode(display_id: u32) -> *mut DisplayMode;
    fn ds_set_mode(display_id: u32, mode_number: u32) -> i32;
    fn ds_configure_display(
        display_id: u32,
        x: i32,
        y: i32,
        rotation: i32,
        mirror_display_id: u32,
        enabled: bool,
    ) -> i32;
    fn ds_free_mode_list(list: *mut DisplayModeList);
    fn ds_free_mode(mode: *mut DisplayMode);
    fn ds_free_string(str: *mut std::os::raw::c_char);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub persistent_id: String, // UUID
    pub contextual_id: u32,
    pub serial: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: u32,
    pub hz: f64,
    pub depth: u32,
    pub scaling: bool,
    pub mode_number: u32,
    pub is_main: bool,
    pub is_mirror: bool,
    pub mirror_of: Option<u32>,
    pub enabled: bool,
    pub display_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub id: String,
    pub mode: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub hz: Option<f64>,
    pub color_depth: Option<u32>,
    pub scaling: Option<bool>,
    pub origin: Option<(i32, i32)>,
    pub degree: Option<u32>,
    pub mirror: Option<String>,
    pub enabled: Option<bool>,
}

pub fn is_display_services_available() -> bool {
    unsafe { ds_is_available() }
}

pub fn get_active_displays() -> Vec<u32> {
    let mut display_count = 0u32;
    unsafe {
        CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut display_count);
        let mut displays = vec![0u32; display_count as usize];
        CGGetActiveDisplayList(display_count, displays.as_mut_ptr(), &mut display_count);
        displays.truncate(display_count as usize);
        displays
    }
}

pub fn get_display_info(display_id: u32) -> Option<DisplayInfo> {
    let bounds = unsafe { CGDisplayBounds(display_id) };
    let mode = get_current_mode(display_id)?;

    let is_main = unsafe { core_graphics::display::CGMainDisplayID() == display_id };
    let is_mirror = unsafe { core_graphics::display::CGDisplayIsInMirrorSet(display_id) != 0 };
    let mirror_of = if is_mirror {
        unsafe {
            let mirror_display = core_graphics::display::CGDisplayMirrorsDisplay(display_id);
            if mirror_display != 0 {
                Some(mirror_display)
            } else {
                None
            }
        }
    } else {
        None
    };

    // Get UUID
    let uuid_ptr = unsafe { ds_get_display_uuid(display_id) };
    let persistent_id = if !uuid_ptr.is_null() {
        let c_str = unsafe { std::ffi::CStr::from_ptr(uuid_ptr) };
        let uuid = c_str.to_string_lossy().to_string();
        unsafe { ds_free_string(uuid_ptr) };
        uuid
    } else {
        format!("{}", display_id)
    };

    // Get display type
    let type_ptr = unsafe { ds_get_display_type(display_id) };
    let display_type = if !type_ptr.is_null() {
        let c_str = unsafe { std::ffi::CStr::from_ptr(type_ptr) };
        let dtype = c_str.to_string_lossy().to_string();
        unsafe { ds_free_string(type_ptr) };
        dtype
    } else {
        "Unknown".to_string()
    };

    Some(DisplayInfo {
        id: display_id,
        persistent_id,
        contextual_id: display_id, // Contextual ID is same as numeric ID
        serial: unsafe { core_graphics::display::CGDisplaySerialNumber(display_id) },
        x: bounds.origin.x as i32,
        y: bounds.origin.y as i32,
        width: mode.width,
        height: mode.height,
        rotation: unsafe { core_graphics::display::CGDisplayRotation(display_id) as u32 },
        hz: mode.refresh_rate,
        depth: mode.depth,
        scaling: mode.is_scaled,
        mode_number: mode.mode_number,
        is_main,
        is_mirror,
        mirror_of,
        enabled: unsafe { core_graphics::display::CGDisplayIsActive(display_id) != 0 },
        display_type,
    })
}

pub fn get_all_modes(display_id: u32) -> Vec<DisplayMode> {
    unsafe {
        let list_ptr = ds_get_all_modes(display_id);
        if list_ptr.is_null() {
            return Vec::new();
        }

        let list = &*list_ptr;
        let modes = std::slice::from_raw_parts(list.modes, list.count).to_vec();

        ds_free_mode_list(list_ptr);
        modes
    }
}

pub fn get_current_mode(display_id: u32) -> Option<DisplayMode> {
    unsafe {
        let mode_ptr = ds_get_current_mode(display_id);
        if mode_ptr.is_null() {
            return None;
        }

        let mode = (*mode_ptr).clone();
        ds_free_mode(mode_ptr);
        Some(mode)
    }
}

pub fn set_display_mode(display_id: u32, mode_number: u32) -> Result<(), String> {
    unsafe {
        let result = ds_set_mode(display_id, mode_number);
        if result == 0 {
            Ok(())
        } else {
            Err(format!("Failed to set display mode: error code {}", result))
        }
    }
}

pub fn configure_display(
    display_id: u32,
    x: Option<i32>,
    y: Option<i32>,
    rotation: Option<u32>,
    mirror_of: Option<u32>,
    enabled: Option<bool>,
) -> Result<(), String> {
    unsafe {
        let result = ds_configure_display(
            display_id,
            x.unwrap_or(-1),
            y.unwrap_or(-1),
            rotation.map(|r| r as i32).unwrap_or(-1),
            mirror_of.unwrap_or(0),
            enabled.unwrap_or(true),
        );
        if result == 0 {
            Ok(())
        } else {
            Err(format!(
                "Failed to configure display: error code {}",
                result
            ))
        }
    }
}

pub fn format_display_command(info: &DisplayInfo) -> String {
    let mut cmd = format!(
        "id:{} res:{}x{} hz:{:.0} color_depth:{} ",
        info.persistent_id, info.width, info.height, info.hz, info.depth
    );

    if info.scaling {
        cmd.push_str("scaling:on ");
    } else {
        cmd.push_str("scaling:off ");
    }

    cmd.push_str(&format!("origin:({},{}) ", info.x, info.y));

    cmd.push_str(&format!("degree:{} ", info.rotation));

    if let Some(mirror_id) = info.mirror_of {
        cmd.push_str(&format!("mirror:{} ", mirror_id));
    }

    if info.enabled {
        cmd.push_str("enabled:true");
    } else {
        cmd.push_str("enabled:false");
    }

    cmd
}

pub fn list_displays() -> String {
    let displays = get_active_displays();
    let mut output = String::new();

    let ds_available = is_display_services_available();
    if !ds_available {
        output.push_str("DisplayServices available: false\n");
        output.push_str("Using CoreGraphics API (official Apple API)\n\n");
    }

    for display_id in displays {
        if let Some(info) = get_display_info(display_id) {
            output.push_str(&format!("Persistent screen id: {}\n", info.persistent_id));
            output.push_str(&format!("Contextual screen id: {}\n", info.contextual_id));
            output.push_str(&format!("Serial screen id: s{}\n", info.serial));
            output.push_str(&format!("Type: {}\n", info.display_type));
            output.push_str(&format!("Resolution: {}x{}\n", info.width, info.height));
            output.push_str(&format!("Hertz: {:.0}\n", info.hz));
            output.push_str(&format!("Color Depth: {}\n", info.depth));
            output.push_str(&format!(
                "Scaling: {}\n",
                if info.scaling { "on" } else { "off" }
            ));
            output.push_str(&format!("Origin: ({},{})", info.x, info.y));
            if info.is_main {
                output.push_str(" - main display");
            }
            output.push('\n');
            output.push_str(&format!("Rotation: {}", info.rotation));
            if info.rotation != 0 {
                output.push_str(" - rotate internal screen example (may crash computer, but will be rotated after rebooting): ");
                output.push_str(&format!(
                    "`macdisp \"id:{} degree:90\"`",
                    info.persistent_id
                ));
            }
            output.push('\n');
            output.push_str(&format!("Enabled: {}\n", info.enabled));

            let modes = get_all_modes(info.id);
            if !modes.is_empty() {
                output.push_str(&format!("Resolutions for rotation {}:\n", info.rotation));
                for (i, mode) in modes.iter().enumerate() {
                    let is_current = mode.mode_number == info.mode_number;
                    output.push_str(&format!(
                        "  mode {}: res:{}x{} hz:{:.0} color_depth:{}",
                        i, mode.width, mode.height, mode.refresh_rate, mode.depth
                    ));
                    if mode.is_scaled {
                        output.push_str(" scaling:on");
                    }
                    if is_current {
                        output.push_str(" <-- current mode");
                    }
                    output.push('\n');
                }
            }

            output.push('\n');
        }
    }

    output.push_str("Execute the command below to set your screens to the current arrangement.");
    output.push_str(" If screen ids are switching, please run `macdisp --help` for info on using contextual or serial ids instead of persistent ids.\n\n");
    output.push_str("macdisp ");

    for display_id in get_active_displays() {
        if let Some(info) = get_display_info(display_id) {
            output.push_str(&format!("\"{}\" ", format_display_command(&info)));
        }
    }

    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_services_available() {
        // This should not panic
        let _available = is_display_services_available();
    }

    #[test]
    fn test_get_active_displays() {
        let displays = get_active_displays();
        // Should have at least one display
        assert!(!displays.is_empty(), "No displays found");
    }

    #[test]
    fn test_display_info() {
        let displays = get_active_displays();
        if let Some(&display_id) = displays.first() {
            let info = get_display_info(display_id);
            assert!(info.is_some(), "Could not get display info");
        }
    }

    #[test]
    fn test_get_modes() {
        let displays = get_active_displays();
        if let Some(&display_id) = displays.first() {
            let modes = get_all_modes(display_id);
            assert!(!modes.is_empty(), "No modes found for display");
        }
    }
}
