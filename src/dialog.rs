use std::path::{Path, PathBuf};

/// Open a new VibeRS window process, optionally starting in `target_dir`.
pub fn spawn_new_window(target_dir: Option<&Path>) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    if let Some(dir) = target_dir {
        cmd.current_dir(dir);
        cmd.arg(dir.as_os_str());
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Open the native Windows 10/11 Explorer Folder Picker dialog (`IFileOpenDialog` with `FOS_PICKFOLDERS`).
#[cfg(target_os = "windows")]
pub fn pick_folder_dialog() -> Option<PathBuf> {
    win_dialog::pick_folder()
}

#[cfg(not(target_os = "windows"))]
pub fn pick_folder_dialog() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
mod win_dialog {
    use std::ffi::c_void;
    use std::path::PathBuf;
    use std::ptr;

    #[repr(C)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const CLSID_FILE_OPEN_DIALOG: GUID = GUID {
        data1: 0xDC1C5A9C,
        data2: 0xE88A,
        data3: 0x4DDE,
        data4: [0xA5, 0xA1, 0x60, 0xF8, 0x2A, 0x20, 0xAE, 0xF7],
    };

    const IID_IFILE_OPEN_DIALOG: GUID = GUID {
        data1: 0xD57C7288,
        data2: 0xD4AD,
        data3: 0x4768,
        data4: [0xBE, 0x02, 0x9D, 0x96, 0x95, 0x32, 0xD9, 0x60],
    };

    const CLSCTX_INPROC_SERVER: u32 = 0x1;
    const FOS_PICKFOLDERS: u32 = 0x0000_0020;
    const FOS_FORCEFILESYSTEM: u32 = 0x0000_0040;
    const SIGDN_FILESYSPATH: u32 = 0x8005_8000;

    #[repr(C)]
    struct IFileOpenDialogVtbl {
        // IUnknown (0..3)
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut IFileOpenDialog) -> u32,
        // IModalWindow (3)
        show: unsafe extern "system" fn(*mut IFileOpenDialog, usize) -> i32,
        // IFileDialog (4..21)
        set_file_types: usize,
        set_file_type_index: usize,
        get_file_type_index: usize,
        advise: usize,
        unadvise: usize,
        set_options: unsafe extern "system" fn(*mut IFileOpenDialog, u32) -> i32,
        get_options: unsafe extern "system" fn(*mut IFileOpenDialog, *mut u32) -> i32,
        set_default_folder: usize,
        set_folder: usize,
        get_folder: usize,
        get_current_selection: usize,
        set_file_name: usize,
        get_file_name: usize,
        set_title: unsafe extern "system" fn(*mut IFileOpenDialog, *const u16) -> i32,
        set_ok_button_label: usize,
        set_file_name_label: usize,
        get_result: unsafe extern "system" fn(*mut IFileOpenDialog, *mut *mut IShellItem) -> i32,
    }

    #[repr(C)]
    struct IFileOpenDialog {
        lp_vtbl: *const IFileOpenDialogVtbl,
    }

    #[repr(C)]
    struct IShellItemVtbl {
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut IShellItem) -> u32,
        bind_to_handler: usize,
        get_parent: usize,
        get_display_name:
            unsafe extern "system" fn(*mut IShellItem, u32, *mut *mut u16) -> i32,
    }

    #[repr(C)]
    struct IShellItem {
        lp_vtbl: *const IShellItemVtbl,
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(pv_reserved: *mut c_void, dw_co_init: u32) -> i32;
        fn CoCreateInstance(
            rclsid: *const GUID,
            p_unk_outer: *mut c_void,
            dw_cls_context: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> i32;
        fn CoTaskMemFree(pv: *mut c_void);
    }

    pub fn pick_folder() -> Option<PathBuf> {
        unsafe {
            let _ = CoInitializeEx(ptr::null_mut(), 0x2); // COINIT_APARTMENTTHREADED

            let mut dialog: *mut IFileOpenDialog = ptr::null_mut();
            let hr = CoCreateInstance(
                &CLSID_FILE_OPEN_DIALOG,
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFILE_OPEN_DIALOG,
                &mut dialog as *mut *mut IFileOpenDialog as *mut *mut c_void,
            );
            if hr < 0 || dialog.is_null() {
                return None;
            }

            let vtbl = &*(*dialog).lp_vtbl;

            let mut options: u32 = 0;
            if (vtbl.get_options)(dialog, &mut options) >= 0 {
                let _ = (vtbl.set_options)(
                    dialog,
                    options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM,
                );
            }

            let title: Vec<u16> = "選擇專案資料夾 (Open Folder)\0".encode_utf16().collect();
            let _ = (vtbl.set_title)(dialog, title.as_ptr());

            let show_hr = (vtbl.show)(dialog, 0);
            if show_hr < 0 {
                let _ = (vtbl.release)(dialog);
                return None;
            }

            let mut shell_item: *mut IShellItem = ptr::null_mut();
            let res_hr = (vtbl.get_result)(dialog, &mut shell_item);
            if res_hr < 0 || shell_item.is_null() {
                let _ = (vtbl.release)(dialog);
                return None;
            }

            let item_vtbl = &*(*shell_item).lp_vtbl;
            let mut path_ptr: *mut u16 = ptr::null_mut();
            let name_hr = (item_vtbl.get_display_name)(
                shell_item,
                SIGDN_FILESYSPATH,
                &mut path_ptr,
            );

            let result = if name_hr >= 0 && !path_ptr.is_null() {
                let mut len = 0usize;
                while *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(path_ptr, len);
                let s = String::from_utf16_lossy(slice);
                CoTaskMemFree(path_ptr as *mut c_void);
                Some(PathBuf::from(s))
            } else {
                None
            };

            let _ = (item_vtbl.release)(shell_item);
            let _ = (vtbl.release)(dialog);
            result
        }
    }
}

