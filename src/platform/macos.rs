use std::ffi::{CStr, c_char, c_void};

use crate::util::{clean, clean_bytes, join_board_name};

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const c_char,
        encoding: u32,
    ) -> *const c_void;
    fn CFStringGetLength(string: *const c_void) -> isize;
    fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    fn CFStringGetCString(
        string: *const c_void,
        buffer: *mut c_char,
        buffer_size: isize,
        encoding: u32,
    ) -> u8;
    fn CFDataGetTypeID() -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFGetTypeID(cf: *const c_void) -> usize;
    fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
    fn CFDataGetLength(data: *const c_void) -> isize;
    fn CFRelease(cf: *const c_void);
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> *const c_void;
    fn IOServiceGetMatchingService(main_port: u32, matching: *const c_void) -> u32;
    fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: *const c_void,
        allocator: *const c_void,
        options: u32,
    ) -> *const c_void;
    fn IORegistryEntryFromPath(main_port: u32, path: *const c_char) -> u32;
    fn IOObjectRelease(object: u32) -> i32;
}

#[link(name = "SystemConfiguration", kind = "framework")]
unsafe extern "C" {
    fn SCDynamicStoreCopyComputerName(
        store: *const c_void,
        name_encoding: *mut u32,
    ) -> *const c_void;
}

#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    fn sysctlbyname(
        name: *const c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> i32;
}

const UTF8: u32 = 0x0800_0100;

pub(crate) fn cpu_name() -> Option<String> {
    sysctl_string(c"machdep.cpu.brand_string")
}

pub(crate) fn cpu_serial() -> Option<String> {
    unique_chip_id().or_else(crate::cpuid::processor_serial)
}

pub(crate) fn computer_name() -> Option<String> {
    sharing_name().or_else(super::hostname)
}

pub(crate) fn board_parts() -> (Option<String>, Option<String>) {
    let Some(service) = platform_expert() else {
        let name = sysctl_string(c"hw.model");
        return (name, None);
    };
    let vendor = registry_string(service, c"manufacturer");
    let product = registry_string(service, c"model")
        .or_else(|| registry_string(service, c"product-name"))
        .or_else(|| registry_string(service, c"board-id"))
        .or_else(|| sysctl_string(c"hw.model"));
    let serial = registry_string(service, c"IOPlatformSerialNumber");
    unsafe {
        IOObjectRelease(service);
    }
    (join_board_name(vendor, product), serial)
}

fn unique_chip_id() -> Option<String> {
    unsafe {
        let entry = IORegistryEntryFromPath(0, c"IODeviceTree:/chosen".as_ptr());
        if entry == 0 {
            return None;
        }
        let bytes = registry_data(entry, c"unique-chip-id");
        IOObjectRelease(entry);
        let bytes = bytes?;
        if bytes.is_empty() || bytes.iter().all(|byte| *byte == 0) {
            return None;
        }
        let mut serial = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            serial.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
            serial.push(char::from(b"0123456789ABCDEF"[(byte & 0xf) as usize]));
        }
        Some(serial)
    }
}

fn registry_data(entry: u32, key: &CStr) -> Option<Vec<u8>> {
    unsafe {
        let cf_key = CFStringCreateWithCString(std::ptr::null(), key.as_ptr(), UTF8);
        if cf_key.is_null() {
            return None;
        }
        let property = IORegistryEntryCreateCFProperty(entry, cf_key, std::ptr::null(), 0);
        CFRelease(cf_key);
        if property.is_null() {
            return None;
        }
        let bytes = if CFGetTypeID(property) == CFDataGetTypeID() {
            let len = CFDataGetLength(property);
            let ptr = CFDataGetBytePtr(property);
            if len > 0 && !ptr.is_null() {
                Some(std::slice::from_raw_parts(ptr, len as usize).to_vec())
            } else {
                None
            }
        } else {
            None
        };
        CFRelease(property);
        bytes
    }
}

fn platform_expert() -> Option<u32> {
    unsafe {
        let matching = IOServiceMatching(c"IOPlatformExpertDevice".as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(0, matching);
        if service == 0 { None } else { Some(service) }
    }
}

fn registry_string(service: u32, key: &CStr) -> Option<String> {
    unsafe {
        let cf_key = CFStringCreateWithCString(std::ptr::null(), key.as_ptr(), UTF8);
        if cf_key.is_null() {
            return None;
        }
        let property = IORegistryEntryCreateCFProperty(service, cf_key, std::ptr::null(), 0);
        CFRelease(cf_key);
        if property.is_null() {
            return None;
        }
        let text = cf_to_string(property);
        CFRelease(property);
        text.as_deref().and_then(clean)
    }
}

fn sharing_name() -> Option<String> {
    unsafe {
        let value = SCDynamicStoreCopyComputerName(std::ptr::null(), std::ptr::null_mut());
        if value.is_null() {
            return None;
        }
        let text = cfstring_to_string(value);
        CFRelease(value);
        text.as_deref().and_then(clean)
    }
}

fn sysctl_string(name: &CStr) -> Option<String> {
    unsafe {
        let mut len = 0usize;
        if sysctlbyname(
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut len,
            std::ptr::null_mut(),
            0,
        ) != 0
            || len == 0
        {
            return None;
        }
        let mut buf = vec![0u8; len];
        if sysctlbyname(
            name.as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return None;
        }
        let n = len.min(buf.len());
        clean_bytes(&buf[..n])
    }
}

fn cf_to_string(value: *const c_void) -> Option<String> {
    unsafe {
        let kind = CFGetTypeID(value);
        if kind == CFStringGetTypeID() {
            cfstring_to_string(value)
        } else if kind == CFDataGetTypeID() {
            let len = CFDataGetLength(value);
            if len < 0 {
                return None;
            }
            let ptr = CFDataGetBytePtr(value);
            if ptr.is_null() {
                return None;
            }
            clean_bytes(std::slice::from_raw_parts(ptr, len as usize))
        } else {
            None
        }
    }
}

fn cfstring_to_string(value: *const c_void) -> Option<String> {
    unsafe {
        let units = CFStringGetLength(value);
        let mut cap = CFStringGetMaximumSizeForEncoding(units, UTF8);
        if cap < 0 {
            return None;
        }
        cap += 1;
        let mut buf = vec![0u8; cap as usize];
        let ok = CFStringGetCString(value, buf.as_mut_ptr().cast(), cap, UTF8);
        if ok == 0 {
            return None;
        }
        clean_bytes(&buf)
    }
}
