#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BaseBoard {
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
}

/// 从 SMBIOS 结构表里找第一块 Type 2（Base Board）。
/// 输入是结构表本身，不含 Windows `RawSMBIOSData` 的 8 字节头。
pub(crate) fn find_baseboard(table: &[u8]) -> Option<BaseBoard> {
    let mut found = None;
    walk(table, |kind, formatted, strings_at| {
        if kind == 2 && formatted.len() >= 8 {
            found = Some(BaseBoard {
                manufacturer: read_string(table, strings_at, formatted[4]),
                product: read_string(table, strings_at, formatted[5]),
                serial: read_string(table, strings_at, formatted[7]),
            });
            return Walk::Stop;
        }
        Walk::Continue
    });
    found
}

/// Type 4（Processor Information）的 Serial Number 字符串。占位符视为没有。
pub(crate) fn find_processor_serial(table: &[u8]) -> Option<String> {
    let mut found = None;
    walk(table, |kind, formatted, strings_at| {
        if kind == 4 && formatted.len() >= 0x21 {
            let serial = read_string(table, strings_at, formatted[0x20]);
            if let Some(serial) = crate::util::useful_id(&serial) {
                found = Some(serial);
                return Walk::Stop;
            }
        }
        Walk::Continue
    });
    found
}

enum Walk {
    Continue,
    Stop,
}

fn walk(table: &[u8], mut visit: impl FnMut(u8, &[u8], usize) -> Walk) {
    let mut offset = 0usize;
    while offset + 4 <= table.len() {
        let kind = table[offset];
        let len = table[offset + 1] as usize;
        if len < 4 || offset.saturating_add(len) > table.len() {
            break;
        }
        if kind == 127 {
            break;
        }
        let strings_at = offset + len;
        let Some(next) = skip_strings(table, strings_at) else {
            break;
        };
        if let Walk::Stop = visit(kind, &table[offset..strings_at], strings_at) {
            break;
        }
        offset = next;
    }
}

fn skip_strings(table: &[u8], mut index: usize) -> Option<usize> {
    if index >= table.len() {
        return None;
    }
    if table[index] == 0 {
        if index + 1 < table.len() && table[index + 1] == 0 {
            return Some(index + 2);
        }
        return None;
    }
    while index < table.len() {
        if table[index] == 0 {
            index += 1;
            if index < table.len() && table[index] == 0 {
                return Some(index + 1);
            }
            continue;
        }
        index += 1;
    }
    None
}

fn read_string(table: &[u8], mut index: usize, which: u8) -> String {
    if which == 0 {
        return String::new();
    }
    let mut number = 1u8;
    while index < table.len() && table[index] != 0 {
        let start = index;
        while index < table.len() && table[index] != 0 {
            index += 1;
        }
        if number == which {
            return String::from_utf8_lossy(&table[start..index])
                .trim()
                .to_string();
        }
        if index >= table.len() {
            break;
        }
        index += 1;
        number = number.saturating_add(1);
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        let mut table = vec![1, 8, 0x00, 0x01, 1, 0, 0, 0];
        table.extend_from_slice(b"System\0\0");
        table.extend_from_slice(&[2, 8, 0x00, 0x02, 1, 2, 3, 4]);
        table.extend_from_slice(b"ASUS\0PRIME B550M\0Rev 1\0MB-SERIAL\0\0");
        table.extend_from_slice(&[127, 4, 0, 0, 0, 0]);
        table
    }

    #[test]
    fn parses_type2_after_another_structure() {
        let board = find_baseboard(&fixture()).unwrap();
        assert_eq!(board.manufacturer, "ASUS");
        assert_eq!(board.product, "PRIME B550M");
        assert_eq!(board.serial, "MB-SERIAL");
    }

    #[test]
    fn malformed_tables_do_not_panic() {
        assert!(find_baseboard(&[]).is_none());
        assert!(find_baseboard(&[2, 100, 0, 0]).is_none());
        assert!(find_baseboard(&[2, 8, 0, 0, 1, 2, 0, 4]).is_none());
        assert!(find_processor_serial(&[4, 8, 0, 0, 0, 0, 0, 0]).is_none());
    }

    #[test]
    fn parses_processor_serial_and_skips_placeholder() {
        let mut table = vec![0u8; 0x21];
        table[0] = 4;
        table[1] = 0x21;
        table[0x20] = 1;
        table.extend_from_slice(b"To Be Filled By O.E.M.\0\0");
        assert!(find_processor_serial(&table).is_none());

        let mut table = vec![0u8; 0x21];
        table[0] = 4;
        table[1] = 0x21;
        table[0x20] = 1;
        table.extend_from_slice(b"CPU-SERIAL-9\0\0");
        assert_eq!(
            find_processor_serial(&table).as_deref(),
            Some("CPU-SERIAL-9")
        );
    }
}
