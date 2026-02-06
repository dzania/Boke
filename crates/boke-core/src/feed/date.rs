use chrono::{DateTime, NaiveDateTime, Utc};

/// Try parsing a date string in multiple common formats used by RSS/Atom feeds.
/// Returns None if no format matches — never panics on bad dates.
pub fn parse_date(input: &str) -> Option<DateTime<Utc>> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // 1. RFC 3339 / ISO 8601: "2024-01-15T10:30:00Z"
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Some(dt.with_timezone(&Utc));
    }

    // 2. RFC 2822: "Mon, 15 Jan 2024 10:30:00 +0000"
    if let Ok(dt) = DateTime::parse_from_rfc2822(input) {
        return Some(dt.with_timezone(&Utc));
    }

    // 3. Common variations with named timezones
    //    Replace common timezone abbreviations with offsets
    let normalized = input
        .replace("GMT", "+0000")
        .replace("EST", "-0500")
        .replace("EDT", "-0400")
        .replace("CST", "-0600")
        .replace("CDT", "-0500")
        .replace("MST", "-0700")
        .replace("MDT", "-0600")
        .replace("PST", "-0800")
        .replace("PDT", "-0700")
        .replace("UTC", "+0000");

    if normalized != input
        && let Ok(dt) = DateTime::parse_from_rfc2822(&normalized) {
            return Some(dt.with_timezone(&Utc));
        }

    // 4. Try naive date/time patterns (assume UTC)
    let naive_formats = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%SZ",
        "%d %b %Y %H:%M:%S",
        "%d %B %Y %H:%M:%S",
        "%a, %d %b %Y %H:%M:%S",
        "%Y-%m-%d",
        "%d/%m/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
    ];

    for fmt in &naive_formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            return Some(naive.and_utc());
        }
    }

    // 5. Date-only fallback
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Some(naive_date.and_hms_opt(0, 0, 0)?.and_utc());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_rfc3339() {
        let dt = parse_date("2024-01-15T10:30:00Z").unwrap();
        assert_eq!(dt.to_rfc3339(), "2024-01-15T10:30:00+00:00");
    }

    #[test]
    fn test_rfc2822() {
        let dt = parse_date("Mon, 15 Jan 2024 10:30:00 +0000").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_rfc2822_with_gmt() {
        let dt = parse_date("Mon, 15 Jan 2024 10:30:00 GMT").unwrap();
        assert_eq!(dt.year(), 2024);
    }

    #[test]
    fn test_date_only() {
        let dt = parse_date("2024-01-15").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_empty_string() {
        assert!(parse_date("").is_none());
    }

    #[test]
    fn test_garbage() {
        assert!(parse_date("not a date").is_none());
    }
}
