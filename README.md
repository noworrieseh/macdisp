# macdisp

A Rust implementation of [displayplacer](https://github.com/jakehilborn/displayplacer) with full compatibility. This tool allows you to configure macOS display settings from the command line.

## Important Note for macOS Sequoia Users

On macOS Sequoia (15.x), Apple has moved the DisplayServices framework into the dyld shared cache and changed/removed many of its APIs. **This is completely fine** - the tool works perfectly using CoreGraphics, which is the official Apple API and provides all necessary functionality:

- ✅ List all displays and their configurations
- ✅ Access all "safe" display modes (60+ modes typically available)
- ✅ Change resolutions and refresh rates
- ✅ Full compatibility with original displayplacer

The only difference is you won't see potentially unstable/unsupported modes that DisplayServices exposed on older macOS versions. For 99.9% of use cases, CoreGraphics provides everything you need.

- **DisplayServices Support**: Accesses private DisplayServices framework for complete display mode information
- **Automatic Fallback**: Falls back to CoreGraphics APIs when DisplayServices is unavailable
- **Full Mode Access**: Can access and set all display modes, including hidden/unsafe ones (via DisplayServices)
- **Compatible CLI**: Drop-in replacement for the original displayplacer syntax

## Installation

### Pre-built Binary (Recommended)

Download the latest release from the [releases page](https://github.com/YOUR_USERNAME/macdisp/releases):

```bash
# Download and install (ARM64 / Apple Silicon)
curl -L https://github.com/YOUR_USERNAME/macdisp/releases/latest/download/macdisp-macos-vX.X.X.tar.gz -o macdisp.tar.gz
tar -xzf macdisp.tar.gz
sudo mv macdisp /usr/local/bin/
sudo chmod +x /usr/local/bin/macdisp

# Or use the universal binary (works on both Intel and Apple Silicon)
curl -L https://github.com/YOUR_USERNAME/macdisp/releases/latest/download/macdisp-universal-vX.X.X.tar.gz -o macdisp.tar.gz
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
git clone <your-repo>
cd macdisp

# Build release version
cargo build --release

# The binary will be at target/release/cd macdisp

```

### Installation

```bash
# Copy to a location in your PATH
sudo cp target/release/cd macdisp /usr/local/bin/
```

## Usage

### List All Displays

```bash
macdisp
# or
macdisp list
```

This shows all connected displays with their current configuration and generates a command to restore the current arrangement.

### Show Available Modes

```bash
macdisp modes <display_id>
```

Shows all available display modes for a specific display, including resolution, refresh rate, bit depth, and whether each mode is safe for hardware.

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
3. **UUID Generation**: Uses IOKit to generate UUIDs since the private API is not publicly available.

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

## License

MIT License - see LICENSE file for details

## Credits

Inspired by [displayplacer](https://github.com/jakehilborn/displayplacer) by Jake Hilborn. Reimplemented in Rust with full feature parity.
