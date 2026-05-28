/// Format a number with comma separators (e.g. 1234567 → "1,234,567").
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Format a byte count as a human-readable string (KB / MB / GB).
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Truncate a path string to at most `max_len` characters,
/// replacing the middle with "..." if needed.
pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    if max_len <= 3 {
        return "...".to_string();
    }
    let keep = max_len - 3;
    let tail = keep / 2;
    let head = keep - tail;
    format!("{}...{}", &path[..head], &path[path.len() - tail..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1_500_000), "1.4 MB");
        assert_eq!(format_file_size(2_500_000_000), "2.3 GB");
    }

    #[test]
    fn test_truncate_path() {
        assert_eq!(truncate_path("short", 20), "short");
        let long = "/very/long/path/to/some/file.glb";
        let truncated = truncate_path(long, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.contains("..."));
    }
}
