use std::fs;

use crate::smbios::{find_baseboard, find_processor_serial};
use crate::util::{clean, clean_bytes, join_board_name, useful_id};

pub(crate) fn cpu_name() -> Option<String> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    for key in ["model name", "Model", "cpu model", "Hardware", "Processor"] {
        for line in text.lines() {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            if name.trim().eq_ignore_ascii_case(key) && useful_cpu(value) {
                return clean(value);
            }
        }
    }
    None
}

pub(crate) fn computer_name() -> Option<String> {
    super::hostname()
}

pub(crate) fn cpu_serial() -> Option<String> {
    cpuinfo_serial()
        .or_else(smbios_processor_serial)
        .or_else(crate::cpuid::processor_serial)
}

pub(crate) fn board_parts() -> (Option<String>, Option<String>) {
    let vendor = read_text("/sys/class/dmi/id/board_vendor");
    let product = read_text("/sys/class/dmi/id/board_name");
    let mut serial = read_text("/sys/class/dmi/id/board_serial");
    let mut name = join_board_name(vendor, product);

    if name.is_none() || serial.is_none() {
        if let Some(board) = read_smbios() {
            if name.is_none() {
                name = join_board_name(clean(&board.manufacturer), clean(&board.product));
            }
            if serial.is_none() {
                serial = clean(&board.serial);
            }
        }
    }

    if name.is_none() {
        name = read_text("/sys/firmware/devicetree/base/model")
            .or_else(|| read_text("/proc/device-tree/model"));
    }
    if serial.is_none() {
        serial = read_text("/sys/firmware/devicetree/base/serial-number")
            .or_else(|| read_text("/proc/device-tree/serial-number"));
    }
    (name, serial)
}

fn useful_cpu(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    !matches!(
        value.to_ascii_lowercase().as_str(),
        "aarch64" | "arm" | "armv7" | "armv8" | "x86_64" | "amd64" | "i386"
    )
}

fn read_text(path: &str) -> Option<String> {
    clean_bytes(&fs::read(path).ok()?)
}

fn read_smbios() -> Option<crate::smbios::BaseBoard> {
    let bytes = fs::read("/sys/firmware/dmi/tables/DMI").ok()?;
    find_baseboard(&bytes)
}

fn cpuinfo_serial() -> Option<String> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in text.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("serial")
            || name.trim().eq_ignore_ascii_case("cpu serial")
        {
            if let Some(serial) = useful_id(value) {
                return Some(serial);
            }
        }
    }
    None
}

fn smbios_processor_serial() -> Option<String> {
    let bytes = fs::read("/sys/firmware/dmi/tables/DMI").ok()?;
    find_processor_serial(&bytes)
}
