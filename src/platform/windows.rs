use std::ffi::c_void;

use crate::smbios::{find_baseboard, find_processor_serial};
use crate::util::{clean, join_board_name};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetComputerNameExW(name_type: i32, buffer: *mut u16, size: *mut u32) -> i32;
    fn GetComputerNameW(buffer: *mut u16, size: *mut u32) -> i32;
    fn GetSystemFirmwareTable(
        signature: u32,
        table_id: u32,
        buffer: *mut u8,
        buffer_size: u32,
    ) -> u32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegOpenKeyExW(
        key: *mut c_void,
        sub_key: *const u16,
        options: u32,
        sam: u32,
        result: *mut *mut c_void,
    ) -> i32;
    fn RegQueryValueExW(
        key: *mut c_void,
        value: *const u16,
        reserved: *mut u32,
        kind: *mut u32,
        data: *mut u8,
        data_len: *mut u32,
    ) -> i32;
    fn RegCloseKey(key: *mut c_void) -> i32;
}

const HKEY_LOCAL_MACHINE: *mut c_void = 0x8000_0002usize as *mut c_void;
const KEY_READ: u32 = 0x20019;
const KEY_WOW64_64KEY: u32 = 0x0100;
const REG_SZ: u32 = 1;
const REG_EXPAND_SZ: u32 = 2;
const COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME: i32 = 5;
const RSMB: u32 = u32::from_be_bytes(*b"RSMB");

pub(crate) fn cpu_name() -> Option<String> {
    reg_sz(
        w("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0"),
        w("ProcessorNameString"),
    )
}

pub(crate) fn computer_name() -> Option<String> {
    dns_hostname().or_else(netbios_name)
}

pub(crate) fn cpu_serial() -> Option<String> {
    let from_smbios =
        firmware_table().and_then(|raw| smbios_payload(&raw).and_then(find_processor_serial));
    from_smbios.or_else(crate::cpuid::processor_serial)
}

pub(crate) fn board_parts() -> (Option<String>, Option<String>) {
    let Some(raw) = firmware_table() else {
        return (None, None);
    };
    let Some(table) = smbios_payload(&raw) else {
        return (None, None);
    };
    let Some(board) = find_baseboard(table) else {
        return (None, None);
    };
    let name = join_board_name(clean(&board.manufacturer), clean(&board.product));
    let serial = clean(&board.serial);
    (name, serial)
}

fn dns_hostname() -> Option<String> {
    unsafe {
        let mut size = 0u32;
        GetComputerNameExW(
            COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME,
            std::ptr::null_mut(),
            &mut size,
        );
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u16; size as usize];
        let ok = GetComputerNameExW(
            COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME,
            buf.as_mut_ptr(),
            &mut size,
        );
        if ok == 0 {
            return None;
        }
        utf16_buf(&buf)
    }
}

fn netbios_name() -> Option<String> {
    unsafe {
        let mut buf = [0u16; 256];
        let mut size = buf.len() as u32;
        if GetComputerNameW(buf.as_mut_ptr(), &mut size) == 0 {
            None
        } else {
            utf16_buf(&buf)
        }
    }
}

fn firmware_table() -> Option<Vec<u8>> {
    unsafe {
        let size = GetSystemFirmwareTable(RSMB, 0, std::ptr::null_mut(), 0);
        if size == 0 || size > 1024 * 1024 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        let written = GetSystemFirmwareTable(RSMB, 0, buf.as_mut_ptr(), size);
        if written == 0 || written as usize > buf.len() {
            return None;
        }
        buf.truncate(written as usize);
        Some(buf)
    }
}

fn smbios_payload(raw: &[u8]) -> Option<&[u8]> {
    if raw.len() < 8 {
        return None;
    }
    let len = u32::from_le_bytes(raw[4..8].try_into().ok()?) as usize;
    let data = raw.get(8..)?;
    Some(&data[..len.min(data.len())])
}

fn reg_sz(sub_key: Vec<u16>, value: Vec<u16>) -> Option<String> {
    unsafe {
        let mut handle: *mut c_void = std::ptr::null_mut();
        let status = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            sub_key.as_ptr(),
            0,
            KEY_READ | KEY_WOW64_64KEY,
            &mut handle,
        );
        if status != 0 || handle.is_null() {
            return None;
        }
        let mut kind = 0u32;
        let mut data_len = 0u32;
        let status = RegQueryValueExW(
            handle,
            value.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            std::ptr::null_mut(),
            &mut data_len,
        );
        if status != 0
            || (kind != REG_SZ && kind != REG_EXPAND_SZ)
            || data_len == 0
            || data_len > 4096
        {
            RegCloseKey(handle);
            return None;
        }
        let mut data = vec![0u8; data_len as usize];
        let status = RegQueryValueExW(
            handle,
            value.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            data.as_mut_ptr(),
            &mut data_len,
        );
        RegCloseKey(handle);
        if status != 0 {
            return None;
        }
        let n = (data_len as usize).min(data.len()) / 2;
        let units: Vec<u16> = data[..n * 2]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        utf16_buf(&units)
    }
}

fn utf16_buf(buf: &[u16]) -> Option<String> {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    clean(&String::from_utf16_lossy(&buf[..end]))
}

fn w(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
