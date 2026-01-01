use clap::{Parser, Subcommand};
use macdisp::{
    configure_display, get_active_displays, get_all_modes, get_current_mode, get_display_info,
    is_display_services_available, list_displays, set_display_mode, DisplayConfig,
};
use std::collections::HashMap;

#[derive(Parser)]
#[command(
    name = "macdisp",
    version = "0.1.0",
    about = "macOS command line utility to configure display settings",
    long_about = "A Rust implementation of displayplacer with full compatibility.\nUses DisplayServices private framework when available, falls back to CoreGraphics."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Display configuration strings (e.g., "id:1 res:1920x1080 hz:60")
    #[arg(trailing_var_arg = true)]
    configs: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all displays and their current configurations
    List,
    /// Show available display modes for a specific display
    Modes {
        /// Display ID
        display_id: u32,
    },
}

fn parse_config(config_str: &str) -> Result<DisplayConfig, String> {
    let mut config = DisplayConfig {
        id: String::new(),
        mode: None,
        resolution: None,
        hz: None,
        color_depth: None,
        scaling: None,
        origin: None,
        degree: None,
        mirror: None,
        enabled: None,
    };

    for part in config_str.split_whitespace() {
        if let Some((key, value)) = part.split_once(':') {
            match key {
                "id" => config.id = value.to_string(),
                "mode" => config.mode = Some(value.to_string()),
                "res" => {
                    if let Some((w, h)) = value.split_once('x') {
                        if let (Ok(width), Ok(height)) = (w.parse(), h.parse()) {
                            config.resolution = Some((width, height));
                        }
                    }
                }
                "hz" => config.hz = value.parse().ok(),
                "color_depth" => config.color_depth = value.parse().ok(),
                "scaling" => config.scaling = Some(value == "on"),
                "origin" => {
                    // Parse (x,y) format
                    let cleaned = value.trim_matches(|c| c == '(' || c == ')');
                    if let Some((x, y)) = cleaned.split_once(',') {
                        if let (Ok(x_val), Ok(y_val)) = (x.parse(), y.parse()) {
                            config.origin = Some((x_val, y_val));
                        }
                    }
                }
                "degree" => config.degree = value.parse().ok(),
                "mirror" => config.mirror = Some(value.to_string()),
                "enabled" => config.enabled = value.parse().ok(),
                _ => {
                    return Err(format!("Unknown configuration key: {}", key));
                }
            }
        }
    }

    if config.id.is_empty() {
        return Err("Display ID is required".to_string());
    }

    Ok(config)
}

fn apply_configuration(configs: Vec<DisplayConfig>) -> Result<(), String> {
    let displays = get_active_displays();
    let display_info: HashMap<u32, _> = displays
        .iter()
        .filter_map(|&id| get_display_info(id).map(|info| (id, info)))
        .collect();

    // Build UUID to ID mapping
    let uuid_to_id: HashMap<String, u32> = display_info
        .iter()
        .map(|(id, info)| (info.persistent_id.clone(), *id))
        .collect();

    for config in configs {
        // Try to parse as numeric ID first, then as UUID
        let display_id = if let Ok(id) = config.id.parse::<u32>() {
            id
        } else if let Some(&id) = uuid_to_id.get(&config.id) {
            id
        } else {
            return Err(format!("Display {} not found", config.id));
        };

        if !display_info.contains_key(&display_id) {
            return Err(format!("Display {} not found", display_id));
        }

        // Handle direct mode number setting
        if let Some(mode_str) = &config.mode {
            let mode_num = mode_str
                .parse::<u32>()
                .map_err(|_| format!("Invalid mode number: {}", mode_str))?;

            set_display_mode(display_id, mode_num)?;

            // Get mode info to display what was set
            if let Some(mode_info) = get_current_mode(display_id) {
                println!(
                    "Set display {} to {}x{} @ {:.0}Hz {} (mode {})",
                    display_id,
                    mode_info.width,
                    mode_info.height,
                    mode_info.refresh_rate,
                    if mode_info.is_scaled {
                        "scaled"
                    } else {
                        "native"
                    },
                    mode_num
                );
            } else {
                println!("Set display {} to mode {}", display_id, mode_num);
            }

            // Skip to next config
            continue;
        }

        // Find and set matching mode
        if config.resolution.is_some() || config.hz.is_some() || config.color_depth.is_some() {
            let modes = get_all_modes(display_id);
            let current = get_current_mode(display_id)
                .ok_or_else(|| format!("Could not get current mode for display {}", display_id))?;

            let target_mode = modes.iter().find(|mode| {
                let res_match = config
                    .resolution
                    .map(|(w, h)| mode.width == w && mode.height == h)
                    .unwrap_or(true);
                let hz_match = config
                    .hz
                    .map(|hz| (mode.refresh_rate - hz).abs() < 0.1)
                    .unwrap_or(true);
                let depth_match = config.color_depth.map(|d| mode.depth == d).unwrap_or(true);
                let scaling_match = config.scaling.map(|s| mode.is_scaled == s).unwrap_or(true);

                res_match && hz_match && depth_match && scaling_match
            });

            if let Some(mode) = target_mode {
                if mode.mode_number != current.mode_number {
                    set_display_mode(display_id, mode.mode_number)?;
                    println!(
                        "Set display {} to {}x{} @ {:.0}Hz {} (mode {})",
                        display_id,
                        mode.width,
                        mode.height,
                        mode.refresh_rate,
                        if mode.is_scaled { "scaled" } else { "native" },
                        mode.mode_number
                    );
                }
            } else {
                return Err(format!(
                    "No matching mode found for display {} with specified parameters",
                    display_id
                ));
            }
        }

        // Handle configuration (mirroring, position, rotation, enable/disable)
        if config.mirror.is_some()
            || config.origin.is_some()
            || config.degree.is_some()
            || config.enabled.is_some()
        {
            let mirror_id = if let Some(mirror_str) = &config.mirror {
                Some(
                    mirror_str
                        .parse::<u32>()
                        .or_else(|_| {
                            uuid_to_id
                                .get(mirror_str.as_str())
                                .copied()
                                .ok_or(format!("Mirror display not found: {}", mirror_str))
                        })
                        .map_err(|e| e.to_string())?,
                )
            } else {
                None
            };

            let (x, y) = config.origin.unzip();

            configure_display(display_id, x, y, config.degree, mirror_id, config.enabled)?;

            if let Some((x, y)) = config.origin {
                println!("Set display {} origin to ({}, {})", display_id, x, y);
            }
            if let Some(degree) = config.degree {
                println!("Set display {} rotation to {}°", display_id, degree);
            }
            if let Some(mirror_id) = mirror_id {
                println!("Set display {} to mirror display {}", display_id, mirror_id);
            }
            if let Some(enabled) = config.enabled {
                println!("Set display {} enabled: {}", display_id, enabled);
            }
        }
    }

    Ok(())
}

fn show_modes(display_id: u32) {
    let modes = get_all_modes(display_id);
    let current = get_current_mode(display_id);

    println!("Available modes for display {}:\n", display_id);
    println!(
        "{:<8} {:<12} {:<10} {:<8} {:<10} {:<6}",
        "Mode #", "Resolution", "Hz", "Depth", "Safe", "Current"
    );
    println!("{:-<70}", "");

    for mode in modes {
        let is_current = current
            .as_ref()
            .map(|c| c.mode_number == mode.mode_number)
            .unwrap_or(false);

        println!(
            "{:<8} {:<12} {:<10.2} {:<8} {:<10} {:<6}",
            mode.mode_number,
            format!("{}x{}", mode.width, mode.height),
            mode.refresh_rate,
            format!("{}-bit", mode.depth),
            if mode.is_safe_for_hardware {
                "yes"
            } else {
                "no"
            },
            if is_current { "*" } else { "" }
        );
    }

    println!("\n* = current mode");
    println!(
        "\nDisplayServices available: {}",
        is_display_services_available()
    );

    if let Some(current) = current {
        println!(
            "Current mode is: {} ({}x{} @ {:.0}Hz)",
            current.mode_number, current.width, current.height, current.refresh_rate
        );
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => {
            print!("{}", list_displays());
        }
        Some(Commands::Modes { display_id }) => {
            show_modes(display_id);
        }
        None => {
            if cli.configs.is_empty() {
                // No arguments, list displays
                print!("{}", list_displays());
            } else {
                // Parse and apply configurations
                let mut configs = Vec::new();
                for config_str in &cli.configs {
                    match parse_config(config_str) {
                        Ok(config) => configs.push(config),
                        Err(e) => {
                            eprintln!("Error parsing configuration: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                if let Err(e) = apply_configuration(configs) {
                    eprintln!("Error applying configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
