# Notch Command Behavior

## Overview
The `macdisp notch` command is designed for MacBook Pro models with a notch. It intelligently switches between display modes to hide or show the notch area.

## Behavior on Different Display Types

### ✅ MacBook Pro with Notch (M2 Pro 14", M3 Pro, etc.)

**Command:**
```bash
macdisp notch hide
```

**Result:**
```
Notch hidden on display 1 (switched to mode 48: 1512x945 @ 120Hz)
```

**What happens:**
- Finds modes with same width (1512), refresh rate (120Hz), color depth (8), and scaling (on)
- Switches to the mode with smaller height (945 instead of 982)
- The display now uses less vertical space, avoiding the notch area

---

### ❌ External Monitor

**Command:**
```bash
macdisp notch hide --display-id 2
```

**Result:**
```
Error: Display 2 is not a MacBook built-in display with a notch
```

**What happens:**
- Detects the display is not a built-in MacBook display
- Refuses to switch modes
- Provides clear error message

---

### ❌ MacBook without Notch (M1 MacBook Air, Intel MacBook Pro, etc.)

**Command:**
```bash
macdisp notch hide
```

**Result:**
```
Error: Display 1 does not appear to have a notch (no alternate height modes found)
```

**What happens:**
- Detects it's a built-in display
- But finds no alternate modes with same specs but different heights
- Refuses to switch modes
- Provides clear error message

---

### ⚠️ Display with Multiple Resolution Options (Edge Case)

**Command:**
```bash
macdisp notch hide --display-id 2
```

**Result:**
```
Warning: Display 2 is not a built-in display. The height difference (200px) may not be notch-related.
Notch hidden on display 2 (switched to mode 15: 2560x1440 @ 60Hz)
```

**What happens:**
- Detects it's not a built-in display
- Finds modes with same width/hz/depth but different heights
- Issues a warning that this may not be notch-related
- Still performs the switch (user may want this for other reasons)

---

## Technical Details

### Detection Logic

1. **Built-in Display Detection:**
   - Checks if display type contains "MacBook" or "built"
   - Uses IOKit to determine if it's the internal display

2. **Notch Mode Detection:**
   - Finds all modes with matching: width, refresh rate, color depth, scaling
   - Checks if there are multiple heights available
   - Typical notch height difference: 30-40 pixels

3. **Safety Checks:**
   - Refuses to operate on external displays without warning
   - Refuses to operate on built-in displays without alternate height modes
   - Warns if height difference seems too large to be notch-related (>100px)

### Mode Selection Algorithm

**For `hide` action:**
1. Filter modes with same width, hz, depth, scaling
2. Sort by height ascending
3. Select mode with height < current height (or smallest if already at minimum)

**For `show` action:**
1. Filter modes with same width, hz, depth, scaling
2. Sort by height ascending  
3. Select mode with height > current height (or largest if already at maximum)

**For `toggle` action:**
1. If at minimum height (notch hidden), switch to maximum (show)
2. Otherwise, switch to minimum (hide)

