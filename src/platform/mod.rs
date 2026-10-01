#[cfg(windows)]
#[path = "windows.rs"]
mod imp;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod imp {
    pub(crate) fn cpu_name() -> Option<String> {
        None
    }

    pub(crate) fn computer_name() -> Option<String> {
        #[cfg(unix)]
        {
            super::hostname()
        }
        #[cfg(not(unix))]
        {
            None
        }
    }

    pub(crate) fn board_parts() -> (Option<String>, Option<String>) {
        (None, None)
    }

    pub(crate) fn cpu_serial() -> Option<String> {
        crate::cpuid::processor_serial()
    }
}

use crate::util::clean;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MachineInfo {
    pub cpu_name: String,
    pub board_name: String,
    pub board_serial: String,
    pub computer_name: String,
    pub cpu_serial: String,
}

pub fn cpu_name() -> Option<String> {
    imp::cpu_name().as_deref().and_then(clean)
}

pub fn board_name() -> Option<String> {
    imp::board_parts().0
}

pub fn board_serial() -> Option<String> {
    imp::board_parts().1
}

pub fn computer_name() -> Option<String> {
    imp::computer_name().as_deref().and_then(clean)
}

pub fn cpu_serial() -> Option<String> {
    imp::cpu_serial()
        .as_deref()
        .and_then(crate::util::useful_id)
}

pub fn query() -> MachineInfo {
    let (board_name, board_serial) = imp::board_parts();
    MachineInfo {
        cpu_name: cpu_name().unwrap_or_default(),
        board_name: board_name.unwrap_or_default(),
        board_serial: board_serial.unwrap_or_default(),
        computer_name: computer_name().unwrap_or_default(),
        cpu_serial: cpu_serial().unwrap_or_default(),
    }
}

#[cfg(unix)]
pub(crate) fn hostname() -> Option<String> {
    use std::ffi::{CStr, c_char};

    unsafe extern "C" {
        fn gethostname(name: *mut c_char, len: usize) -> i32;
    }

    let mut buf = [0u8; 256];
    let rc = unsafe { gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if rc != 0 {
        return None;
    }
    let text = CStr::from_bytes_until_nul(&buf).ok()?.to_string_lossy();
    clean(&text)
}
