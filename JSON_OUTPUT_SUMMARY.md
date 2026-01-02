# JSON Output Implementation Summary

## Overview
Added `--json` flag to the `list` and `modes` commands to output structured JSON data for easy scripting and integration with other tools.

## Changes Made

### 1. Code Modifications

#### `src/lib.rs`
- Added `Serialize` and `Deserialize` derives to `DisplayMode` struct
- Enabled JSON serialization for display mode information

#### `src/main.rs`
- Added `--json` flag to `Commands::List` enum variant
- Added `--json` flag to `Commands::Modes` enum variant
- Implemented `list_displays_json()` function
- Modified `show_modes()` function to accept `json: bool` parameter
- Updated command handling in `main()` to respect JSON flags
- Added `serde_json` import for JSON serialization

### 2. New Functions

#### `list_displays_json()`
Returns JSON array of all display information:
```rust
fn list_displays_json() -> String
```

Outputs:
```json
[
  {
    "id": 1,
    "persistent_id": "UUID",
    "contextual_id": 1,
    "serial": 123456,
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

#### `show_modes()` with JSON support
Modified to accept `json: bool` parameter:
```rust
fn show_modes(display_id: u32, json: bool)
```

When `json = true`, outputs:
```json
{
  "display_id": 1,
  "current_mode": { ... },
  "available_modes": [ ... ],
  "display_services_available": true
}
```

## Usage Examples

### List Command

**Regular output:**
```bash
$ macdisp list
Persistent screen id: 37D8832A-2D66-02CA-B9F7-8F30A301B230
Contextual screen id: 1
Serial screen id: s4251086178
Type: MacBook built in screen
...
```

**JSON output:**
```bash
$ macdisp list --json
[
  {
    "id": 1,
    "persistent_id": "37D8832A-2D66-02CA-B9F7-8F30A301B230",
    ...
  }
]
```

### Modes Command

**Regular output:**
```bash
$ macdisp modes 1
Available modes for display 1:

Mode #   Resolution   Hz         Depth    Safe       Current
----------------------------------------------------------------------
0        960x600      120.00     8-bit    yes
...
```

**JSON output:**
```bash
$ macdisp modes 1 --json
{
  "display_id": 1,
  "current_mode": {
    "width": 1512,
    "height": 945,
    ...
  },
  "available_modes": [...]
}
```

## Scripting Examples

### Get Main Display ID
```bash
MAIN_ID=$(macdisp list --json | python3 -c "import sys, json; displays = json.load(sys.stdin); print([d['persistent_id'] for d in displays if d['is_main']][0])")
```

### Count Available Modes
```bash
MODE_COUNT=$(macdisp modes 1 --json | python3 -c "import sys, json; data = json.load(sys.stdin); print(len(data['available_modes']))")
```

### Get Current Resolution
```bash
RESOLUTION=$(macdisp list --json | python3 -c "import sys, json; d = json.load(sys.stdin)[0]; print(f\"{d['width']}x{d['height']}\")")
```

### Find All 120Hz Modes
```bash
macdisp modes 1 --json | python3 -c "import sys, json; data = json.load(sys.stdin); [print(f\"Mode {m['mode_number']}: {m['width']}x{m['height']} @ {m['refresh_rate']}Hz\") for m in data['available_modes'] if m['refresh_rate'] == 120.0]"
```

## JSON Schema

### Display Object (from `list --json`)
```typescript
{
  id: number,                    // Numeric display ID
  persistent_id: string,         // UUID that persists across reboots
  contextual_id: number,         // Context-dependent ID
  serial: number,                // Display serial number
  x: number,                     // X position in screen space
  y: number,                     // Y position in screen space
  width: number,                 // Current width in pixels
  height: number,                // Current height in pixels
  rotation: number,              // Rotation in degrees (0, 90, 180, 270)
  hz: number,                    // Refresh rate
  depth: number,                 // Color depth in bits
  scaling: boolean,              // HiDPI/Retina scaling enabled
  mode_number: number,           // Current mode number
  is_main: boolean,              // Is this the main display
  is_mirror: boolean,            // Is this display mirroring another
  mirror_of: number | null,      // Display ID being mirrored (if applicable)
  enabled: boolean,              // Display is enabled
  display_type: string           // Display type description
}
```

### Mode Object (from `modes --json`)
```typescript
{
  display_id: number,
  current_mode: {
    width: number,
    height: number,
    refresh_rate: number,
    depth: number,
    mode_number: number,
    is_stretched: boolean,
    is_interlaced: boolean,
    is_tv_mode: boolean,
    is_safe_for_hardware: boolean,
    is_scaled: boolean
  },
  available_modes: [
    {
      width: number,
      height: number,
      refresh_rate: number,
      depth: number,
      mode_number: number,
      is_stretched: boolean,
      is_interlaced: boolean,
      is_tv_mode: boolean,
      is_safe_for_hardware: boolean,
      is_scaled: boolean
    },
    ...
  ],
  display_services_available: boolean
}
```

## Testing Results

### JSON Validation
✓ List JSON is valid (verified with Python json.tool)
✓ Modes JSON is valid (verified with Python json.tool)

### Backward Compatibility
✓ Regular output unchanged when --json not specified
✓ All existing commands still work as expected

### Scripting Integration
✓ All documented Python examples tested and working
✓ JSON can be piped to jq, Python, or other JSON processors
✓ Output is properly formatted (pretty-printed)

## Build Status
- ✅ No warnings
- ✅ No errors
- ✅ Full backwards compatibility maintained
- ✅ JSON output properly formatted

## Files Modified
1. `src/lib.rs` - Added Serialize/Deserialize to DisplayMode
2. `src/main.rs` - Added JSON support to list and modes commands
3. `README.md` - Added comprehensive JSON documentation

## Dependencies
- Already had `serde_json = "1.0"` in Cargo.toml
- No new dependencies required

## Use Cases

### Automation Scripts
```bash
#!/bin/bash
# Save current display configuration
CONFIG=$(macdisp list --json)
echo "$CONFIG" > display_config.json

# Restore later
MAIN_ID=$(echo "$CONFIG" | jq -r '.[0].persistent_id')
WIDTH=$(echo "$CONFIG" | jq -r '.[0].width')
HEIGHT=$(echo "$CONFIG" | jq -r '.[0].height')
macdisp "id:$MAIN_ID res:${WIDTH}x${HEIGHT}"
```

### Monitoring
```bash
# Monitor display changes
while true; do
  macdisp list --json > /tmp/displays_current.json
  if ! diff -q /tmp/displays_current.json /tmp/displays_previous.json; then
    echo "Display configuration changed!"
  fi
  mv /tmp/displays_current.json /tmp/displays_previous.json
  sleep 5
done
```

### Integration with Other Tools
```bash
# Export to Ansible/Terraform variable format
macdisp list --json | jq '{displays: .}' > displays.auto.tfvars.json

# Generate configuration from JSON
macdisp list --json | python3 -c "
import sys, json
displays = json.load(sys.stdin)
for d in displays:
    print(f'macdisp \"id:{d[\"persistent_id\"]} res:{d[\"width\"]}x{d[\"height\"]} hz:{d[\"hz\"]}\"')
"
```

