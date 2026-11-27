//! Launch at Windows startup.
//!
//! Two install flavours need two mechanisms:
//! - MSIX (Microsoft Store) package: writes to `HKCU\...\Run` from a packaged
//!   process are virtualized into the package's private registry hive, so
//!   Windows never sees them at logon. Packaged apps must declare a
//!   `windows.startupTask` extension in `AppxManifest.xml` and toggle it with
//!   the `Windows.ApplicationModel.StartupTask` API.
//! - MSI/NSIS install: a `HKCU\...\Run` value whose executable path is quoted
//!   (the default install folder contains spaces), plus the Task Manager
//!   "StartupApproved" flag.

/// Must match the `TaskId` of the `desktop:StartupTask` in `AppxManifest.xml`.
pub const STARTUP_TASK_ID: &str = "SmartPrtScrStartup";

/// Name of the `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value
/// (same name as the one previously written by tauri-plugin-autostart, so an
/// old unquoted entry is overwritten).
pub const RUN_VALUE_NAME: &str = "smart-prtscr";

/// Argument passed when launched at logon.
pub const STARTUP_ARG: &str = "--autostart";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Backend {
    /// MSIX package: `Windows.ApplicationModel.StartupTask`.
    StartupTask,
    /// Classic install: `HKCU\...\Run` registry value.
    RunKey,
}

pub fn backend_for(is_packaged: bool) -> Backend {
    if is_packaged {
        Backend::StartupTask
    } else {
        Backend::RunKey
    }
}

/// Command line stored in the Run key: executable path always quoted.
pub fn run_key_command(exe_path: &str, args: &[&str]) -> String {
    let mut cmd = format!("\"{}\"", exe_path.trim_matches('"'));
    for arg in args {
        cmd.push(' ');
        cmd.push_str(arg);
    }
    cmd
}

/// `StartupApproved\Run` binary value: first byte even (02) = enabled,
/// odd (03) = disabled by the user in Task Manager.
pub fn startup_approved_is_enabled(bytes: &[u8]) -> bool {
    bytes.first().map(|b| b % 2 == 0).unwrap_or(true)
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::{RegKey, RegValue};

    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const APPROVED_KEY: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

    pub fn is_packaged() -> bool {
        use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
        use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
        let mut len: u32 = 0;
        let rc = unsafe { GetCurrentPackageFullName(&mut len, windows::core::PWSTR::null()) };
        rc != APPMODEL_ERROR_NO_PACKAGE
    }

    // ---- MSIX StartupTask -------------------------------------------------

    /// WinRT calls need an initialized apartment: run them on a dedicated MTA thread.
    fn with_startup_task<T: Send + 'static>(
        f: impl FnOnce(&windows::ApplicationModel::StartupTask) -> windows::core::Result<T> + Send + 'static,
    ) -> Result<T, String> {
        std::thread::spawn(move || -> Result<T, String> {
            use windows::ApplicationModel::StartupTask;
            use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
            unsafe {
                let _ = RoInitialize(RO_INIT_MULTITHREADED);
            }
            let task = StartupTask::GetAsync(&windows::core::HSTRING::from(STARTUP_TASK_ID))
                .and_then(|op| op.get())
                .map_err(|e| format!("StartupTask '{}' unavailable: {}", STARTUP_TASK_ID, e))?;
            f(&task).map_err(|e| e.to_string())
        })
        .join()
        .map_err(|_| "StartupTask thread panicked".to_string())?
    }

    fn task_enabled(state: windows::ApplicationModel::StartupTaskState) -> bool {
        use windows::ApplicationModel::StartupTaskState;
        state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy
    }

    fn startup_task_set(enabled: bool) -> Result<bool, String> {
        use windows::ApplicationModel::StartupTaskState;
        let state = with_startup_task(move |task| {
            if enabled {
                task.RequestEnableAsync()?.get()
            } else {
                task.Disable()?;
                task.State()
            }
        })?;
        if enabled && state == StartupTaskState::DisabledByUser {
            return Err("Le démarrage automatique a été désactivé dans le Gestionnaire des tâches \
                        (onglet Démarrage) : réactivez-le à cet endroit."
                .to_string());
        }
        if enabled && state == StartupTaskState::DisabledByPolicy {
            return Err("Le démarrage automatique est bloqué par une stratégie système.".to_string());
        }
        Ok(task_enabled(state))
    }

    // ---- Run key ------------------------------------------------------------

    pub(crate) fn run_key_is_enabled() -> Result<bool, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let present = hkcu
            .open_subkey_with_flags(RUN_KEY, KEY_READ)
            .map_err(|e| e.to_string())?
            .get_value::<String, _>(RUN_VALUE_NAME)
            .is_ok();
        let approved = hkcu
            .open_subkey_with_flags(APPROVED_KEY, KEY_READ)
            .ok()
            .and_then(|k| k.get_raw_value(RUN_VALUE_NAME).ok())
            .map(|v| startup_approved_is_enabled(&v.bytes))
            .unwrap_or(true);
        Ok(present && approved)
    }

    pub(crate) fn run_key_set(enabled: bool) -> Result<bool, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run = hkcu
            .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
            .map_err(|e| e.to_string())?;
        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let cmd = run_key_command(&exe.to_string_lossy(), &[STARTUP_ARG]);
            run.set_value(RUN_VALUE_NAME, &cmd).map_err(|e| e.to_string())?;
            // Clear a previous "disabled in Task Manager" flag.
            if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED_KEY, KEY_SET_VALUE) {
                let mut bytes = vec![0u8; 12];
                bytes[0] = 0x02;
                let _ = approved.set_raw_value(
                    RUN_VALUE_NAME,
                    &RegValue { vtype: winreg::enums::RegType::REG_BINARY, bytes },
                );
            }
        } else {
            match run.delete_value(RUN_VALUE_NAME) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
            if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED_KEY, KEY_SET_VALUE) {
                let _ = approved.delete_value(RUN_VALUE_NAME);
            }
        }
        run_key_is_enabled()
    }

    // ---- Public API -----------------------------------------------------------

    pub fn is_enabled() -> Result<bool, String> {
        match backend_for(is_packaged()) {
            Backend::StartupTask => with_startup_task(|task| task.State()).map(task_enabled),
            Backend::RunKey => run_key_is_enabled(),
        }
    }

    /// Returns the effective state after the change.
    pub fn set_enabled(enabled: bool) -> Result<bool, String> {
        match backend_for(is_packaged()) {
            Backend::StartupTask => startup_task_set(enabled),
            Backend::RunKey => run_key_set(enabled),
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn is_enabled() -> Result<bool, String> {
        Ok(false)
    }

    pub fn set_enabled(_enabled: bool) -> Result<bool, String> {
        Err("Launch at startup is only supported on Windows".to_string())
    }
}

pub use imp::{is_enabled, set_enabled};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_app_uses_startup_task_not_run_key() {
        assert_eq!(backend_for(true), Backend::StartupTask);
        assert_eq!(backend_for(false), Backend::RunKey);
    }

    #[test]
    fn run_key_command_quotes_paths_with_spaces() {
        assert_eq!(
            run_key_command(r"C:\Program Files\Smart PrtScr\smart-prtscr.exe", &[STARTUP_ARG]),
            r#""C:\Program Files\Smart PrtScr\smart-prtscr.exe" --autostart"#
        );
        // Already-quoted input is not double-quoted.
        assert_eq!(run_key_command(r#""C:\a b\x.exe""#, &[]), r#""C:\a b\x.exe""#);
    }

    #[test]
    fn startup_approved_flag_is_decoded() {
        assert!(startup_approved_is_enabled(&[0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
        assert!(!startup_approved_is_enabled(&[0x03, 0, 0, 0, 0x10, 0x20, 0, 0, 0, 0, 0, 0]));
        assert!(startup_approved_is_enabled(&[]));
    }

    /// Real registry round-trip for the MSI/NSIS flavour. Writes then removes the
    /// `smart-prtscr` value under HKCU\...\Run. Run explicitly:
    /// `cargo test --lib -- --ignored run_key_roundtrip`
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "modifies HKCU Run of the current user"]
    fn run_key_roundtrip_on_this_machine() {
        use winreg::enums::HKEY_CURRENT_USER;
        let run = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .unwrap();
        assert!(
            run.get_value::<String, _>(RUN_VALUE_NAME).is_err(),
            "precondition: no existing '{}' Run value",
            RUN_VALUE_NAME
        );

        assert!(imp::run_key_set(true).unwrap());
        let value: String = run.get_value(RUN_VALUE_NAME).unwrap();
        println!("Run value written: {}", value);
        let exe = std::env::current_exe().unwrap();
        assert_eq!(value, run_key_command(&exe.to_string_lossy(), &[STARTUP_ARG]));
        assert!(value.starts_with('"'));
        assert!(imp::run_key_is_enabled().unwrap());

        assert!(!imp::run_key_set(false).unwrap());
        assert!(run.get_value::<String, _>(RUN_VALUE_NAME).is_err());
        assert!(!imp::run_key_is_enabled().unwrap());
    }

    #[test]
    fn msix_manifest_declares_the_startup_task() {
        let manifest = include_str!("../AppxManifest.xml");
        assert!(
            manifest.contains(r#"Category="windows.startupTask""#),
            "AppxManifest.xml must declare a windows.startupTask extension, otherwise the \
             Store build can never start with Windows"
        );
        assert!(
            manifest.contains(&format!(r#"TaskId="{}""#, STARTUP_TASK_ID)),
            "StartupTask TaskId in AppxManifest.xml must equal STARTUP_TASK_ID"
        );
        assert!(manifest.contains(r#"Executable="smart-prtscr.exe""#));
    }
}
