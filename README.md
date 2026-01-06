# macdisp

A Rust implementation of [displayplacer](https://github.com/jakehilborn/displayplacer) with full compatibility. This tool allows you to configure macOS display settings from the command line.

## Installation

### Homebrew

```bash
# Add the tap
brew tap noworrieseh/brews

# Install macdisp
brew install macdisp
```

Or as a one-liner:

```bash
brew install noworrieseh/brews/macdisp
```

### MacPorts

```bash
sudo port install macdisp
```

### crates.io

```bash
cargo install macdisp
```

### Pre-built Binary

Download the latest release from the [releases page](https://github.com/noworrieseh/macdisp/releases):

```bash
# Download and install (ARM64 / Apple Silicon)
curl -L https://github.com/noworrieseh/macdisp/releases/latest/download/macdisp-macos-vX.X.X.tar.gz -o macdisp.tar.gz
tar -xzf macdisp.tar.gz
sudo mv macdisp /usr/local/bin/
sudo chmod +x /usr/local/bin/macdisp

# Or use the universal binary (works on both Intel and Apple Silicon)
curl -L https://github.com/noworrieseh/macdisp/releases/latest/download/macdisp-universal-vX.X.X.tar.gz -o macdisp.tar.gz
tar -xzf macdisp.tar.gz
sudo mv macdisp-universal /usr/local/bin/macdisp
sudo chmod +x /usr/local/bin/macdisp
```

### Build from Source

### Prerequisites

- Rust toolchain (1.70+)
- macOS (tested on 10.13+)
- Xcode Command Line Tools

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/noworrieseh/macdisp.git
cd macdisp

# Build release version
cargo build --release

# The binary will be at target/release/macdisp

```

### Installation

```bash
# Copy to a location in your PATH
sudo cp target/release/macdisp /usr/local/bin/
```

## Usage

### List All Displays

```bash
macdisp
# or
macdisp list

# Output as JSON
macdisp list --json
```

This shows all connected displays with their current configuration and generates a command to restore the current arrangement.

The `--json` flag outputs the display information in JSON format, which is useful for scripting and integration with other tools.

### Show Available Modes

```bash
macdisp modes <display_id>

# Output as JSON
macdisp modes <display_id> --json
```

Shows all available display modes for a specific display, including resolution, refresh rate, bit depth, and whether each mode is safe for hardware.

The `--json` flag outputs the mode information in JSON format, including the current mode and all available modes with detailed properties.

### Hide/Show the Notch (MacBook Pro)

```bash
# Hide the notch (switch to smaller height mode)
macdisp notch hide

# Show the notch (switch to larger height mode)
macdisp notch show

# Toggle between hiding and showing the notch
macdisp notch toggle

# Specify a display ID (defaults to main display)
macdisp notch hide --display-id 1
```

On MacBook Pro models with a notch, this command switches between display modes with different heights while maintaining the same width, refresh rate, and scaling settings. The "hide" mode uses a slightly smaller vertical resolution that doesn't extend into the notch area, while "show" mode uses the full screen height including the notch area.

**Note:** If you run this command on a display without a notch (external monitors, older MacBooks, etc.), the tool will detect this and provide an appropriate error message:

- For non-built-in displays: "Display X is not a MacBook built-in display with a notch"
- For built-in displays without alternate height modes: "Display X does not appear to have a notch (no alternate height modes found)"

This prevents accidentally switching to inappropriate display modes on non-notch displays.

### Configure Displays

```bash
macdisp "id:1 res:1920x1080 hz:60 color_depth:32 origin:(0,0) degree:0 enabled:true"
```

#### Configuration Parameters

- `id:<number>` - Display ID (required)
- `res:<width>x<height>` - Resolution
- `hz:<refresh_rate>` - Refresh rate in Hz
- `color_depth:<bits>` - Color depth (8, 16, or 32)
- `origin:(<x>,<y>)` - Display position
- `degree:<rotation>` - Rotation (0, 90, 180, 270)
- `mirror:<display_id>` - Mirror another display
- `enabled:<true|false>` - Enable/disable display

### Examples

```bash
# Set display 1 to 1920x1080 @ 60Hz
macdisp "id:1 res:1920x1080 hz:60"

# Configure multiple displays
macdisp \
  "id:1 res:2560x1440 hz:60 origin:(0,0) enabled:true" \
  "id:2 res:1920x1080 hz:60 origin:(2560,0) enabled:true"

# Set a display to a specific mode number
macdisp "id:1 mode:123"

# Get display information as JSON for scripting
macdisp list --json

# Get all available modes as JSON
macdisp modes 1 --json
```

### Notch Management

```bash
# Hide the notch on current display
macdisp notch hide

# Show the notch on current display
macdisp notch show

# Toggle notch visibility
macdisp notch toggle

# Target a specific display
macdisp notch hide --display-id 1
```

The notch command intelligently finds display modes with matching specifications (width, refresh rate, color depth, scaling) but different heights to hide or show the notch area on compatible MacBook Pro displays. The command includes safety checks to prevent use on non-notch displays, providing clear error messages when run on external monitors or MacBooks without a notch.

## JSON Output

Both the `list` and `modes` commands support JSON output for easy integration with scripts and other tools.

### List Command JSON Output

```bash
macdisp list --json
```

Returns an array of display objects:

```json
[
    {
        "id": 1,
        "persistent_id": "37D8832A-2D66-02CA-B9F7-8F30A301B230",
        "contextual_id": 1,
        "serial": 4251086178,
        "x": 0,
        "y": 0,
        "width": 1512,
        "height": 945,
        "rotation": 0,
        "hz": 120.0,
        "depth": 8,
        "scaling": true,
        "mode_number": 48,
        "is_main": true,
        "is_mirror": false,
        "mirror_of": null,
        "enabled": true,
        "display_type": "MacBook built in screen"
    }
]
```

### Modes Command JSON Output

```bash
macdisp modes 1 --json
```

Returns detailed mode information:

```json
{
    "display_id": 1,
    "current_mode": {
        "width": 1512,
        "height": 945,
        "refresh_rate": 120.0,
        "depth": 8,
        "mode_number": 48,
        "is_stretched": false,
        "is_interlaced": false,
        "is_tv_mode": false,
        "is_safe_for_hardware": true,
        "is_scaled": true
    },
    "available_modes": [
        {
            "width": 960,
            "height": 600,
            "refresh_rate": 120.0,
            "depth": 8,
            "mode_number": 0,
            "is_stretched": false,
            "is_interlaced": false,
            "is_tv_mode": false,
            "is_safe_for_hardware": true,
            "is_scaled": true
        }
    ],
    "display_services_available": true
}
```

### Scripting Examples

```bash
# Get the persistent ID of the main display
MAIN_ID=$(macdisp list --json | python3 -c "import sys, json; displays = json.load(sys.stdin); print([d['persistent_id'] for d in displays if d['is_main']][0])")

# Count available modes for display 1
MODE_COUNT=$(macdisp modes 1 --json | python3 -c "import sys, json; data = json.load(sys.stdin); print(len(data['available_modes']))")

# Get current resolution
RESOLUTION=$(macdisp list --json | python3 -c "import sys, json; d = json.load(sys.stdin)[0]; print(f\"{d['width']}x{d['height']}\")")

# Find all 120Hz modes
macdisp modes 1 --json | python3 -c "import sys, json; data = json.load(sys.stdin); [print(f\"Mode {m['mode_number']}: {m['width']}x{m['height']} @ {m['refresh_rate']}Hz\") for m in data['available_modes'] if m['refresh_rate'] == 120.0]"
```

## Architecture

### Components

1. **Objective-C Helper** (`src/objc/display_services.m`)
    - Dynamically loads DisplayServices private framework
    - Provides C API for Rust to call
    - Gracefully handles missing framework
    - Implements UUID generation and display type detection

2. **Rust Library** (`src/lib.rs`)
    - Safe Rust wrappers around C APIs
    - Display information and mode management
    - Configuration parsing and application

3. **CLI Tool** (`src/main.rs`)
    - Command-line interface using clap
    - Compatible with displayplacer syntax

### DisplayServices vs CoreGraphics

The tool attempts to use the DisplayServices private framework first, which provides:

- Access to all display modes (including HiDPI/scaled modes)
- More detailed mode information (scaled, interlaced flags)
- Mode switching for scaled/Retina resolutions

If DisplayServices is unavailable, it falls back to CoreGraphics, which:

- Only shows "safe" native resolution modes
- Has less detailed mode information
- Works on all macOS versions

## Compatibility

### macOS Versions

- macOS 10.13+ (High Sierra and later)
- DisplayServices API changed in Sequoia (15.x) - fully supported
- CoreGraphics fallback ensures universal compatibility

### Original displayplacer

This implementation provides full CLI compatibility with the original displayplacer. All configuration strings work identically.

## Known Limitations

1. **Display Rotation**: Reading rotation works, but setting rotation via public APIs is not available. Rotation requires system reboot to take effect.
2. **Enable/Disable**: The main display cannot be disabled via public APIs.

## Development

### Project Structure

```
macdisp/
├── Cargo.toml           # Rust dependencies
├── build.rs             # Build script (compiles Obj-C)
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Core library
│   └── objc/
│       ├── display_services.h  # C header
│       └── display_services.m  # Obj-C implementation
└── README.md
```

### Adding Features

The DisplayServices wrapper can be extended to support additional features:

- Enhanced rotation support (requires private APIs)
- HDR and color space configuration
- More detailed display properties

## Contributing

Contributions welcome! Areas for improvement:

- Enhanced private API support for rotation
- Better error handling and user feedback
- Support for additional display features (HDR, color profiles)
- Automated testing

## Important Note for macOS Sequoia Users

On macOS Sequoia (15.x), Apple has moved the DisplayServices framework into the dyld shared cache and changed/removed many of its APIs. The tool works perfectly using CoreGraphics, which is the official Apple API and provides all necessary functionality:

- ✅ List all displays and their configurations
- ✅ Access all "safe" display modes (60+ modes typically available)
- ✅ Change resolutions and refresh rates
- ✅ Full compatibility with original displayplacer

The only difference is you won't see potentially unstable/unsupported modes that DisplayServices exposed on older macOS versions. For 99.9% of use cases, CoreGraphics provides everything you need.

- **DisplayServices Support**: Accesses private DisplayServices framework for complete display mode information
- **Automatic Fallback**: Falls back to CoreGraphics APIs when DisplayServices is unavailable
- **Full Mode Access**: Can access and set all display modes, including hidden/unsafe ones (via DisplayServices)
- **Compatible CLI**: Drop-in replacement for the original displayplacer syntax

## License

MIT License - see LICENSE file for details

## Credits

Inspired by [displayplacer](https://github.com/jakehilborn/displayplacer) by Jake Hilborn. Reimplemented in Rust with full feature parity.
