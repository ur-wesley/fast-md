//! File associations and OS shell integration service.
//! Enables Windows Explorer suggestion, "Open With" list, and Default Apps capability.

#[cfg(windows)]
mod win_impl {
    use std::env;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegCreateKeyW,
        RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    };
    use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

    const PROG_ID: &str = "FastMD.Document";
    const APP_NAME: &str = "Fast-MD";
    const APP_DESC: &str = "Lightning-fast native desktop Markdown & MDX viewer";
    const SUPPORTED_EXTS: &[&str] = &[".md", ".markdown", ".mdx", ".mdown"];

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    struct RegKey(HKEY);

    impl Drop for RegKey {
        fn drop(&mut self) {
            if self.0 != 0 as HKEY {
                unsafe {
                    RegCloseKey(self.0);
                }
            }
        }
    }

    fn create_key(hkey: HKEY, subkey: &str) -> Option<RegKey> {
        let wide_subkey = to_wide(subkey);
        let mut result_key: HKEY = 0 as HKEY;
        let status = unsafe {
            RegCreateKeyW(
                hkey,
                wide_subkey.as_ptr(),
                &raw mut result_key,
            )
        };
        if status == ERROR_SUCCESS {
            Some(RegKey(result_key))
        } else {
            None
        }
    }

    fn set_value_str(key: &RegKey, name: Option<&str>, value: &str) -> bool {
        let wide_value = to_wide(value);
        let wide_name = name.map(to_wide);
        let name_ptr = wide_name.as_ref().map_or(ptr::null(), std::vec::Vec::as_ptr);
        let byte_len = u32::try_from(wide_value.len().saturating_mul(std::mem::size_of::<u16>()))
            .unwrap_or(0);

        let status = unsafe {
            RegSetValueExW(
                key.0,
                name_ptr,
                0,
                REG_SZ,
                wide_value.as_ptr().cast::<u8>(),
                byte_len,
            )
        };
        status == ERROR_SUCCESS
    }

    fn delete_tree(hkey: HKEY, subkey: &str) -> bool {
        let wide_key = to_wide(subkey);
        let status = unsafe { RegDeleteTreeW(hkey, wide_key.as_ptr()) };
        status == ERROR_SUCCESS
    }

    /// Register Fast-MD in Windows Explorer and Default Apps.
    #[must_use]
    pub fn register_associations() -> bool {
        let Ok(exe_path) = env::current_exe() else {
            return false;
        };
        let exe_str = exe_path.to_string_lossy();
        let open_command = format!("\"{exe_str}\" \"%1\"");
        let icon_str = format!("\"{exe_str}\",0");

        // 1. Register ProgID: HKCU\Software\Classes\FastMD.Document
        if let Some(key) = create_key(HKEY_CURRENT_USER, "Software\\Classes\\FastMD.Document") {
            set_value_str(&key, None, "Markdown Document");
            set_value_str(&key, Some("FriendlyTypeName"), "Markdown Document");
        }
        if let Some(key) =
            create_key(HKEY_CURRENT_USER, "Software\\Classes\\FastMD.Document\\DefaultIcon")
        {
            set_value_str(&key, None, &icon_str);
        }
        if let Some(key) = create_key(
            HKEY_CURRENT_USER,
            "Software\\Classes\\FastMD.Document\\shell\\open\\command",
        ) {
            set_value_str(&key, None, &open_command);
            set_value_str(&key, Some("FriendlyAppName"), APP_NAME);
        }

        // 2. Register Application entries: HKCU\Software\Classes\Applications\fmd.exe & fast-md.exe
        let app_exe_name = exe_path
            .file_name()
            .map_or_else(|| "fmd.exe".to_string(), |n| n.to_string_lossy().to_string());
        let app_names = ["fmd.exe", "fast-md.exe", &app_exe_name];
        for name in &app_names {
            let app_key_path = format!("Software\\Classes\\Applications\\{name}");
            if let Some(key) = create_key(HKEY_CURRENT_USER, &app_key_path) {
                set_value_str(&key, Some("FriendlyAppName"), APP_NAME);
            }
            if let Some(key) = create_key(HKEY_CURRENT_USER, &format!("{app_key_path}\\DefaultIcon")) {
                set_value_str(&key, None, &icon_str);
            }
            if let Some(key) =
                create_key(HKEY_CURRENT_USER, &format!("{app_key_path}\\shell\\open\\command"))
            {
                set_value_str(&key, None, &open_command);
            }
            if let Some(key) =
                create_key(HKEY_CURRENT_USER, &format!("{app_key_path}\\SupportedTypes"))
            {
                for ext in SUPPORTED_EXTS {
                    set_value_str(&key, Some(ext), "");
                }
            }
        }

        // 3. Register OpenWithProgids & OpenWithList for each extension
        for ext in SUPPORTED_EXTS {
            if let Some(key) =
                create_key(HKEY_CURRENT_USER, &format!("Software\\Classes\\{ext}\\OpenWithProgids"))
            {
                set_value_str(&key, Some(PROG_ID), "");
            }
            for name in &app_names {
                if let Some(key) = create_key(
                    HKEY_CURRENT_USER,
                    &format!("Software\\Classes\\{ext}\\OpenWithList\\{name}"),
                ) {
                    set_value_str(&key, None, "");
                }
            }
        }

        // 4. Register Windows App Paths so `fmd` and `fast-md` are directly callable from CMD / PowerShell / Run Dialog
        let exe_dir = exe_path
            .parent()
            .map_or_else(String::new, |p| p.to_string_lossy().to_string());
        for app_bin in &["fmd.exe", "fast-md.exe"] {
            let app_path_key = format!("Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{app_bin}");
            if let Some(key) = create_key(HKEY_CURRENT_USER, &app_path_key) {
                set_value_str(&key, None, &exe_str);
                if !exe_dir.is_empty() {
                    set_value_str(&key, Some("Path"), &exe_dir);
                }
            }
        }

        // 5. Register Capabilities for Windows Default Apps settings
        if let Some(key) = create_key(HKEY_CURRENT_USER, "Software\\FastMD\\Capabilities") {
            set_value_str(&key, Some("ApplicationName"), APP_NAME);
            set_value_str(&key, Some("ApplicationDescription"), APP_DESC);
        }
        if let Some(key) = create_key(
            HKEY_CURRENT_USER,
            "Software\\FastMD\\Capabilities\\FileAssociations",
        ) {
            for ext in SUPPORTED_EXTS {
                set_value_str(&key, Some(ext), PROG_ID);
            }
        }
        if let Some(key) = create_key(HKEY_CURRENT_USER, "Software\\RegisteredApplications") {
            set_value_str(&key, Some("FastMD"), "Software\\FastMD\\Capabilities");
        }

        // 6. Notify Windows Shell of association change
        unsafe {
            SHChangeNotify(SHCNE_ASSOCCHANGED.cast_signed(), SHCNF_IDLIST, ptr::null(), ptr::null());
        }

        true
    }

    /// Unregister Fast-MD from Windows Explorer.
    #[must_use]
    pub fn unregister_associations() -> bool {
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\Classes\\FastMD.Document");
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\Classes\\Applications\\fmd.exe");
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\Classes\\Applications\\fast-md.exe");
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\fmd.exe");
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\fast-md.exe");
        let _ = delete_tree(HKEY_CURRENT_USER, "Software\\FastMD");

        for ext in SUPPORTED_EXTS {
            let wide_key = to_wide(&format!("Software\\Classes\\{ext}\\OpenWithProgids"));
            let mut hkey: HKEY = 0 as HKEY;
            let status =
                unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide_key.as_ptr(), 0, KEY_SET_VALUE, &raw mut hkey) };
            if status == ERROR_SUCCESS {
                let wide_val = to_wide(PROG_ID);
                unsafe {
                    RegDeleteValueW(hkey, wide_val.as_ptr());
                    RegCloseKey(hkey);
                }
            }
        }

        unsafe {
            SHChangeNotify(SHCNE_ASSOCCHANGED.cast_signed(), SHCNF_IDLIST, ptr::null(), ptr::null());
        }

        true
    }

    /// Check if Fast-MD is registered in Windows registry.
    #[must_use]
    pub fn is_registered() -> bool {
        let wide_key = to_wide("Software\\Classes\\FastMD.Document\\shell\\open\\command");
        let mut hkey: HKEY = 0 as HKEY;
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide_key.as_ptr(), 0, KEY_READ, &raw mut hkey) };
        if status == ERROR_SUCCESS {
            unsafe { RegCloseKey(hkey) };
            true
        } else {
            false
        }
    }
}

/// Register file associations with the OS.
#[must_use]
pub fn register_file_associations() -> bool {
    #[cfg(windows)]
    {
        win_impl::register_associations()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Unregister file associations from the OS.
#[must_use]
pub fn unregister_file_associations() -> bool {
    #[cfg(windows)]
    {
        win_impl::unregister_associations()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Check if file associations are registered.
#[must_use]
pub fn is_file_associations_registered() -> bool {
    #[cfg(windows)]
    {
        win_impl::is_registered()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Open the OS default apps settings UI (Windows Settings / macOS System Settings).
pub fn open_default_apps_settings() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "ms-settings:defaultapps"])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.general")
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_association_registration_lifecycle() {
        // Registering should succeed without panic
        let registered = register_file_associations();
        assert!(registered);

        // Verification check
        let is_reg = is_file_associations_registered();
        assert!(is_reg);
    }
}

