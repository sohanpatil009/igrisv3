# File Explorer Implementation for File Share

## Overview
The file share panel needs TWO views:
1. **Device List View** - Shows available devices (already working)
2. **File Explorer View** - Shows after connecting to a device

## Implementation Plan

### State Structure
```rust
pub enum PanelView {
    DeviceList,
    FileExplorer,
}

pub struct FileItem {
    path: PathBuf,
    name: String,
    is_dir: bool,
    size: u64,
    selected: bool,  // For checkbox
}

pub struct FileSharePanelState {
    current_view: PanelView,
    connected_device: Option<DeviceInfo>,
    current_path: PathBuf,
    files: Vec<FileItem>,
    selected_files: Vec<PathBuf>,
}
```

### File Explorer UI Features

**Header:**
- Back button to return to device list
- Current path breadcrumb
- Connected device name
- Selected files count (e.g., "3 files selected")
- Send button (enabled when files selected)

**File List:**
- Checkbox for each file/folder
- File icon (📁 for folders, 📄 for files)
- File name
- File size (formatted: KB, MB, GB)
- Click folder to navigate into it
- Click checkbox to select/deselect

**Actions:**
- Navigate up (..)
- Select all / Deselect all buttons
- Send selected files button
- Cancel button

### User Flow
1. User clicks device card → Connect
2. View switches to FileExplorer
3. Load files from current directory
4. User checks files they want to send
5. Click "Send" button
6. Files transfer to connected device
7. Show progress/success message

### Key Functions Needed
- `load_directory_files(path)` - Read directory contents
- `toggle_file_selection(path)` - Toggle checkbox
- `navigate_to(path)` - Change directory
- `send_selected_files()` - Transfer files
- `format_size(bytes)` - Human-readable sizes

## Next Steps
The file is too large for single write. Need to:
1. Create base structure with imports
2. Add helper functions
3. Add device list render function
4. Add file explorer render function  
5. Add main component

Would you like me to create this in multiple smaller files or use a different approach?
