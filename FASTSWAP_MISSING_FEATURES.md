# FastSwap Missing Features Comparison

## Comparison with Reference Implementation (localshare-desktop)

After analyzing the reference implementation, here are the key differences:

---

## ✅ Already Implemented in FastSwap

1. **Server & Client** - Complete LocalSend v2.0 protocol
2. **Device Discovery** - Full subnet scanning (254 IPs)
3. **Three-way Handshake** - Prepare → Confirm → Upload
4. **Progress Tracking Models** - FileProgress, TransferProgress, ProgressTracker
5. **Port Conflict Handling** - Tries ports 53317-53326
6. **Streaming Upload** - 64KB chunks with progress updates
7. **Token Verification** - Security tokens for uploads
8. **Error Handling** - Comprehensive error messages

---

## ❌ Missing in FastSwap UI

### 1. **Folder Selection** (CRITICAL)
**Reference has**: Two buttons - "Select Files" and "Select Folder"
**FastSwap has**: Only file picker (no folder support)

**Impact**: Users cannot send entire folders, must select files individually

**Fix Needed**:
```rust
// Add folder picker button
let select_folder = move |_| {
    spawn(async move {
        match rfd::FileDialog::new().pick_folder() {
            Some(folder_path) => {
                // Recursively collect all files
                fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                files.push(path);
                            } else if path.is_dir() {
                                collect_files(&path, files); // Recursive
                            }
                        }
                    }
                }
            }
        }
    });
};
```

---

### 2. **Real-time Progress Updates** (CRITICAL)
**Reference has**: Progress update loop that polls every 100ms
**FastSwap has**: Progress UI but no polling mechanism

**Impact**: Progress bars don't update during transfer

**Fix Needed**:
```rust
// Add progress polling effect
use_effect(move || {
    spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Poll progress tracker and update UI
            if let Some(session_id) = current_transfer() {
                let tracker = progress_tracker.read().await;
                if let Some(progress) = tracker.get(&session_id) {
                    // Update active_transfers signal
                    let mut transfers = active_transfers();
                    for (i, file_progress) in progress.files.iter().enumerate() {
                        if i < transfers.len() {
                            transfers[i] = file_progress.clone();
                        }
                    }
                    active_transfers.set(transfers);
                }
            }
        }
    });
});
```

---

### 3. **Server Lifecycle Management** (IMPORTANT)
**Reference has**: Server started in use_effect on mount
**FastSwap has**: Server started in main.rs (global)

**Impact**: Server is always running (good), but UI doesn't show server status

**Current State**: Actually better - server is global and always available

---

### 4. **File Selection UI** (NICE TO HAVE)
**Reference has**: Shows selected files list with sizes before sending
**FastSwap has**: Opens picker → immediately starts transfer

**Impact**: No chance to review files before sending

**Fix Needed**:
```rust
// Add selected_files state
let mut selected_files = use_signal(|| Vec::<PathBuf>::new());

// Show selected files list
if !selected_files().is_empty() {
    div {
        h4 { "Selected Files:" }
        for file in selected_files() {
            div { "{file.file_name()}" }
        }
        button {
            onclick: move |_| {
                // Send files
            },
            "Send to Device"
        }
    }
}
```

---

### 5. **Cancel Transfer** (NICE TO HAVE)
**Reference has**: Cancel button during transfer
**FastSwap has**: No cancel functionality

**Impact**: Cannot abort ongoing transfers

**Fix Needed**:
```rust
// Add cancel button
button {
    onclick: move |_| {
        if let Some(session_id) = current_transfer() {
            spawn(async move {
                let mut tracker = progress_tracker.write().await;
                if let Some(progress) = tracker.get_mut(&session_id) {
                    progress.cancel();
                }
            });
        }
    },
    "Cancel Transfer"
}
```

---

### 6. **Transfer History** (NICE TO HAVE)
**Reference has**: Completed transfers list
**FastSwap has**: Only shows active transfers

**Impact**: No record of past transfers

---

## 🔧 Priority Fixes

### High Priority (Must Fix)
1. **Real-time Progress Updates** - Progress bars don't work without this
2. **Folder Selection** - Essential for usability

### Medium Priority (Should Fix)
3. **File Selection Review** - Better UX before sending
4. **Cancel Transfer** - Important for large files

### Low Priority (Nice to Have)
5. **Transfer History** - Useful but not critical
6. **Better Error Display** - Current errors are basic

---

## 📋 Implementation Checklist

### Phase 1: Critical Fixes (1-2 hours)
- [ ] Add progress polling loop (100ms interval)
- [ ] Connect progress tracker to UI signals
- [ ] Add folder selection button
- [ ] Implement recursive folder scanning

### Phase 2: UX Improvements (1 hour)
- [ ] Add selected files list display
- [ ] Add "Clear Selection" button
- [ ] Add "Send" button (separate from device click)
- [ ] Show total size of selected files

### Phase 3: Advanced Features (2 hours)
- [ ] Add cancel transfer button
- [ ] Implement transfer history
- [ ] Add transfer speed graph
- [ ] Add estimated time remaining

---

## 🎯 Recommended Next Steps

1. **Fix Progress Updates** (30 min)
   - Add polling effect
   - Connect to progress tracker
   - Update UI signals

2. **Add Folder Support** (30 min)
   - Add folder picker button
   - Implement recursive scanning
   - Update UI to show folder info

3. **Improve File Selection** (30 min)
   - Add selected files list
   - Add clear button
   - Separate send action

4. **Test Everything** (30 min)
   - Test with small files
   - Test with large files
   - Test with folders
   - Test progress updates

---

## 📊 Feature Comparison Table

| Feature | Reference | FastSwap | Status |
|---------|-----------|----------|--------|
| File Transfer | ✅ | ✅ | Complete |
| Device Discovery | ✅ | ✅ | Complete |
| Progress Tracking | ✅ | ⚠️ | Models exist, UI not connected |
| Folder Selection | ✅ | ❌ | Missing |
| Multiple Files | ✅ | ✅ | Complete |
| Real-time Progress | ✅ | ❌ | Missing |
| Cancel Transfer | ✅ | ❌ | Missing |
| Transfer History | ✅ | ❌ | Missing |
| Port Conflict | ✅ | ✅ | Complete |
| Streaming Upload | ✅ | ✅ | Complete |
| Error Handling | ✅ | ✅ | Complete |

---

## 🚀 Quick Win: Fix Progress Updates

This is the most critical missing piece. Here's the minimal fix:

```rust
// In FastSwapPanel component, add this effect:
use_effect(move || {
    spawn(async move {
        loop {
            async_std::task::sleep(std::time::Duration::from_millis(200)).await;
            
            // Update transfers from global progress tracker
            // (This requires exposing the progress tracker globally)
        }
    });
});
```

The challenge is that the progress tracker is created inside `send_files_to_device` and not accessible globally. We need to either:
1. Make progress tracker global (like FASTSWAP_MANAGER)
2. Pass it through signals
3. Use a different progress tracking approach

---

## 💡 Conclusion

FastSwap has **excellent core implementation** (protocol, streaming, error handling) but needs:
1. **Progress UI connection** - Most critical
2. **Folder support** - Essential for UX
3. **Better file selection flow** - Nice to have

The reference implementation is more polished in the UI/UX department, but FastSwap's core is solid. With 2-3 hours of work, FastSwap can match or exceed the reference.
