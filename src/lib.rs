//! 读取 CPU 名称、主板名称、主板序列号和计算机名。
//!
//! 动态库导出 C ABI，见 `include/hwinfo.h`。Rust 调用方直接使用本模块的函数。

mod cpuid;
mod ffi;
mod platform;
#[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
mod smbios;
mod util;

pub use ffi::{
    ABI_VERSION, ERR_INTERNAL, ERR_NULL, ERR_SMALL, ERR_UNAVAILABLE, HAS_BOARD_NAME,
    HAS_BOARD_SERIAL, HAS_COMPUTER_NAME, HAS_CPU_NAME, HAS_CPU_SERIAL, HwMachine, OK, STR_MAX,
};
pub use platform::{
    MachineInfo, board_name, board_serial, computer_name, cpu_name, cpu_serial, query,
};

#[cfg(test)]
mod tests {
    use std::ffi::c_char;

    use super::*;
    use crate::ffi::{hwinfo_cpu_name, hwinfo_machine_size, hwinfo_query};

    #[test]
    fn abi_constants_match_header() {
        assert_eq!(ABI_VERSION, 2);
        assert_eq!(STR_MAX, 256);
        assert_eq!(hwinfo_machine_size(), STR_MAX * 5);
        assert_eq!(OK, 0);
        assert_eq!(ERR_NULL, -1);
        assert_eq!(ERR_SMALL, -2);
        assert_eq!(ERR_UNAVAILABLE, -3);
        assert_eq!(ERR_INTERNAL, -4);
    }

    #[test]
    fn string_api_reports_length_and_rejects_short_buffers() {
        let mut needed = 0usize;
        let rc = hwinfo_cpu_name(std::ptr::null_mut(), 0, &mut needed);
        if rc == ERR_UNAVAILABLE {
            return;
        }
        assert_eq!(rc, ERR_SMALL);
        assert!(needed > 0);

        let mut tiny = [0 as c_char; 1];
        let rc = hwinfo_cpu_name(tiny.as_mut_ptr(), tiny.len(), &mut needed);
        assert_eq!(rc, ERR_SMALL);

        let mut buf = vec![0 as c_char; needed + 1];
        let rc = hwinfo_cpu_name(buf.as_mut_ptr(), buf.len(), &mut needed);
        assert_eq!(rc, OK);
        assert_eq!(buf[needed], 0);
    }

    #[test]
    fn query_null_is_rejected() {
        assert_eq!(hwinfo_query(std::ptr::null_mut()), ERR_NULL);
    }

    #[test]
    fn live_machine_is_readable() {
        let info = query();
        let mut raw = std::mem::MaybeUninit::<HwMachine>::zeroed();
        let mask = hwinfo_query(raw.as_mut_ptr());
        assert!(mask > 0, "at least one field should be readable");
        assert!(!info.cpu_name.is_empty(), "cpu name");
        assert!(!info.computer_name.is_empty(), "computer name");
        assert!(mask & HAS_CPU_NAME != 0);
        assert!(mask & HAS_COMPUTER_NAME != 0);
        #[cfg(target_os = "macos")]
        {
            assert!(!info.board_name.is_empty(), "board name");
            assert!(!info.board_serial.is_empty(), "board serial");
            assert!(!info.cpu_serial.is_empty(), "cpu serial");
            assert_eq!(
                mask,
                HAS_CPU_NAME
                    | HAS_BOARD_NAME
                    | HAS_BOARD_SERIAL
                    | HAS_COMPUTER_NAME
                    | HAS_CPU_SERIAL
            );
        }
    }
}
