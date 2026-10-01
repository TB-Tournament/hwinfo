pub(crate) fn clean(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_matches('\0');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn clean_bytes(bytes: &[u8]) -> Option<String> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    clean(&String::from_utf8_lossy(&bytes[..end]))
}

/// 主板名称用厂商和产品拼出来。产品名已经以厂商开头时不再重复。
pub(crate) fn join_board_name(vendor: Option<String>, product: Option<String>) -> Option<String> {
    let vendor = vendor.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let product = product.as_deref().map(str::trim).filter(|s| !s.is_empty());
    match (vendor, product) {
        (None, None) => None,
        (Some(vendor), None) => Some(vendor.to_string()),
        (None, Some(product)) => Some(product.to_string()),
        (Some(vendor), Some(product)) => {
            if product
                .to_ascii_lowercase()
                .starts_with(&vendor.to_ascii_lowercase())
            {
                Some(product.to_string())
            } else {
                Some(format!("{vendor} {product}"))
            }
        }
    }
}

/// 丢掉空值和固件占位符。全 0、全 F 也视为没有序列号。
pub(crate) fn useful_id(value: &str) -> Option<String> {
    let text = clean(value)?;
    let token: String = text
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && !matches!(*c, '-' | ':' | '.'))
        .flat_map(|c| c.to_lowercase())
        .collect();
    const PLACEHOLDERS: &[&str] = &[
        "none",
        "notspecified",
        "notavailable",
        "notapplicable",
        "defaultstring",
        "tobefilledbyoem",
        "systemserialnumber",
        "oem",
        "unknown",
        "na",
        "invalid",
        "null",
        "unspecified",
        "notset",
        "empty",
    ];
    if token.is_empty()
        || PLACEHOLDERS.contains(&token.as_str())
        || token.chars().all(|c| c == '0')
        || token.chars().all(|c| c == 'f')
    {
        None
    } else {
        Some(text)
    }
}

pub(crate) fn fit(value: &str, max: usize) -> &str {
    if value.len() <= max {
        return value;
    }
    let mut end = max;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[cfg(test)]
mod tests {
    use super::useful_id;

    #[test]
    fn rejects_blank_placeholder_and_zero_ids() {
        assert!(useful_id("  ").is_none());
        assert!(useful_id("To Be Filled By O.E.M.").is_none());
        assert!(useful_id("0000-0000-0000").is_none());
        assert!(useful_id("FFFFFFFF").is_none());
        assert_eq!(useful_id("  CPU-1 ").as_deref(), Some("CPU-1"));
    }
}
