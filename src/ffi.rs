use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use crate::platform::{self, MachineInfo};
use crate::util::fit;

pub const ABI_VERSION: u32 = 2;
pub const STR_MAX: usize = 256;
pub const OK: i32 = 0;
pub const ERR_NULL: i32 = -1;
pub const ERR_SMALL: i32 = -2;
pub const ERR_UNAVAILABLE: i32 = -3;
pub const ERR_INTERNAL: i32 = -4;
pub const HAS_CPU_NAME: i32 = 0x1;
pub const HAS_BOARD_NAME: i32 = 0x2;
pub const HAS_BOARD_SERIAL: i32 = 0x4;
pub const HAS_COMPUTER_NAME: i32 = 0x8;
pub const HAS_CPU_SERIAL: i32 = 0x10;

#[repr(C)]
pub struct HwMachine {
    pub cpu_name: [c_char; STR_MAX],
    pub board_name: [c_char; STR_MAX],
    pub board_serial: [c_char; STR_MAX],
    pub computer_name: [c_char; STR_MAX],
    pub cpu_serial: [c_char; STR_MAX],
}

fn guard(body: impl FnOnce() -> i32) -> i32 {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(code) => code,
        Err(_) => ERR_INTERNAL,
    }
}

fn export_string(
    value: Option<String>,
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    let Some(value) = value else {
        if !out_len.is_null() {
            unsafe { *out_len = 0 };
        }
        return ERR_UNAVAILABLE;
    };
    if !out_len.is_null() {
        unsafe { *out_len = value.len() };
    }
    if buf.is_null() {
        return if buf_len == 0 { ERR_SMALL } else { ERR_NULL };
    }
    if buf_len <= value.len() {
        return ERR_SMALL;
    }
    unsafe {
        ptr::copy_nonoverlapping(value.as_ptr(), buf.cast(), value.len());
        *buf.add(value.len()) = 0;
    }
    OK
}

fn write_field(dst: &mut [c_char], value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let text = fit(value, dst.len().saturating_sub(1));
    unsafe {
        ptr::copy_nonoverlapping(text.as_ptr(), dst.as_mut_ptr().cast(), text.len());
    }
    dst[text.len()] = 0;
    true
}

fn store(out: *mut HwMachine, info: &MachineInfo) -> i32 {
    unsafe {
        ptr::write_bytes(out, 0, 1);
        let slot = &mut *out;
        let mut mask = 0;
        if write_field(&mut slot.cpu_name, &info.cpu_name) {
            mask |= HAS_CPU_NAME;
        }
        if write_field(&mut slot.board_name, &info.board_name) {
            mask |= HAS_BOARD_NAME;
        }
        if write_field(&mut slot.board_serial, &info.board_serial) {
            mask |= HAS_BOARD_SERIAL;
        }
        if write_field(&mut slot.computer_name, &info.computer_name) {
            mask |= HAS_COMPUTER_NAME;
        }
        if write_field(&mut slot.cpu_serial, &info.cpu_serial) {
            mask |= HAS_CPU_SERIAL;
        }
        mask
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_abi_version() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_machine_size() -> usize {
    size_of::<HwMachine>()
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_query(out: *mut HwMachine) -> i32 {
    guard(|| {
        if out.is_null() {
            return ERR_NULL;
        }
        store(out, &platform::query())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_cpu_name(buf: *mut c_char, buf_len: usize, out_len: *mut usize) -> i32 {
    guard(|| export_string(platform::cpu_name(), buf, buf_len, out_len))
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_cpu_serial(buf: *mut c_char, buf_len: usize, out_len: *mut usize) -> i32 {
    guard(|| export_string(platform::cpu_serial(), buf, buf_len, out_len))
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_board_name(buf: *mut c_char, buf_len: usize, out_len: *mut usize) -> i32 {
    guard(|| export_string(platform::board_name(), buf, buf_len, out_len))
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_board_serial(
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    guard(|| export_string(platform::board_serial(), buf, buf_len, out_len))
}

#[unsafe(no_mangle)]
pub extern "C" fn hwinfo_computer_name(
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    guard(|| export_string(platform::computer_name(), buf, buf_len, out_len))
}
