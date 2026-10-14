//! Utility functions for HTML processing.

use url::Url;

/// Attributes whose whole value is a single URL. The `data-*` variants are where lazy-loading
/// scripts keep the real image URL; resolving them lets the frontend promote them to `src`.
const URL_ATTRIBUTES: [&str; 6] = [
    "src",
    "href",
    "poster",
    "data-src",
    "data-lazy-src",
    "data-original",
];

/// Attributes whose value is a comma-separated list of `url [descriptor]` candidates.
const SRCSET_ATTRIBUTES: [&str; 2] = ["srcset", "data-srcset"];

/// URL prefixes that must never be joined with a base URL.
const UNRESOLVED_PREFIXES: [&str; 5] = ["#", "data:", "javascript:", "mailto:", "tel:"];

/// Rewrites relative URLs in `html` to absolute ones, resolved against `base`.
///
/// Handles `src`, `href`, `poster`, `srcset` and their lazy-loading `data-*` variants, with
/// double-quoted, single-quoted or unquoted values and optional whitespace around `=`. Attribute
/// names are matched whole, so `data-src` is never mistaken for `src`. Comment bodies are copied
/// unchanged. Absolute URLs and fragment, `data:`, `javascript:`, `mailto:` and `tel:` URLs are
/// left as they are.
///
/// Runs in a single pass and allocates one output string of roughly the input size.
pub fn resolve_relative_urls(html: &str, base: &Url) -> String {
    let bytes = html.as_bytes();
    let mut rewriter = Rewriter {
        html,
        output: String::with_capacity(html.len() + html.len() / 8),
        copied_until: 0,
    };

    let mut cursor = 0;
    while let Some(offset) = html[cursor..].find('<') {
        let tag_start = cursor + offset;
        if html[tag_start..].starts_with("<!--") {
            match html[tag_start + 4..].find("-->") {
                Some(end) => cursor = tag_start + 4 + end + 3,
                None => break,
            }
            continue;
        }

        let name_start = tag_start + 1;
        let starts_tag = bytes
            .get(name_start)
            .is_some_and(|byte| byte.is_ascii_alphabetic());
        cursor = if starts_tag {
            rewriter.rewrite_tag(name_start, base)
        } else {
            name_start
        };
        assert!(cursor > tag_start);
    }

    rewriter.finish()
}

struct Rewriter<'a> {
    html: &'a str,
    output: String,
    /// Everything in `html` before this index has already been written to `output`.
    copied_until: usize,
}

impl Rewriter<'_> {
    /// Walks the attributes of the tag whose name starts at `name_start`, rewriting URL values.
    /// Returns the index just past the tag.
    fn rewrite_tag(&mut self, name_start: usize, base: &Url) -> usize {
        let bytes = self.html.as_bytes();
        let length = bytes.len();
        let mut index = name_start;
        while index < length && !is_name_terminator(bytes[index]) {
            index += 1;
        }

        loop {
            while index < length && (bytes[index].is_ascii_whitespace() || bytes[index] == b'/') {
                index += 1;
            }
            if index >= length {
                return length;
            }
            if bytes[index] == b'>' {
                return index + 1;
            }

            let attribute_start = index;
            while index < length && !is_name_terminator(bytes[index]) && bytes[index] != b'=' {
                index += 1;
            }
            if index == attribute_start {
                // A stray `=` with no attribute name. Skip it so the loop always advances.
                index += 1;
                continue;
            }
            let attribute_name = &self.html[attribute_start..index];

            let mut after_name = index;
            while after_name < length && bytes[after_name].is_ascii_whitespace() {
                after_name += 1;
            }
            if after_name >= length || bytes[after_name] != b'=' {
                // Attribute without a value, such as `<input disabled>`.
                continue;
            }

            let mut value_start = after_name + 1;
            while value_start < length && bytes[value_start].is_ascii_whitespace() {
                value_start += 1;
            }
            let Some((value_range, next_index)) = value_bounds(bytes, value_start) else {
                return length;
            };
            index = next_index;
            self.rewrite_value(attribute_name, value_range, base);
        }
    }

    fn rewrite_value(&mut self, attribute_name: &str, value: (usize, usize), base: &Url) {
        let (value_start, value_end) = value;
        assert!(value_start <= value_end);
        assert!(value_start >= self.copied_until);

        let raw_value = &self.html[value_start..value_end];
        let resolved = if is_one_of(attribute_name, &URL_ATTRIBUTES) {
            resolve_url(raw_value, base)
        } else if is_one_of(attribute_name, &SRCSET_ATTRIBUTES) {
            resolve_srcset(raw_value, base)
        } else {
            None
        };

        if let Some(resolved) = resolved {
            self.output
                .push_str(&self.html[self.copied_until..value_start]);
            self.output.push_str(&resolved);
            self.copied_until = value_end;
        }
    }

    fn finish(mut self) -> String {
        self.output.push_str(&self.html[self.copied_until..]);
        self.output
    }
}

fn is_name_terminator(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'>' || byte == b'/'
}

fn is_one_of(attribute_name: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| attribute_name.eq_ignore_ascii_case(candidate))
}

/// Returns the `(start, end)` byte range of the attribute value at `value_start`, excluding
/// quotes, and the index just past the value. Returns `None` for an unterminated quoted value.
fn value_bounds(bytes: &[u8], value_start: usize) -> Option<((usize, usize), usize)> {
    let length = bytes.len();
    match bytes.get(value_start) {
        Some(&quote @ (b'"' | b'\'')) => {
            let content_start = value_start + 1;
            let closing_offset = bytes[content_start..]
                .iter()
                .position(|&byte| byte == quote)?;
            let content_end = content_start + closing_offset;
            Some(((content_start, content_end), content_end + 1))
        }
        _ => {
            let mut end = value_start;
            while end < length && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
                end += 1;
            }
            Some(((value_start, end), end))
        }
    }
}

/// Resolves one URL against `base`. Returns `None` when the value should stay unchanged.
fn resolve_url(value: &str, base: &Url) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let is_unresolved = UNRESOLVED_PREFIXES.iter().any(|prefix| {
        trimmed
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
    });
    if is_unresolved || Url::parse(trimmed).is_ok() {
        return None;
    }
    base.join(trimmed).ok().map(String::from)
}

/// Resolves every candidate URL in a `srcset` value, keeping width and density descriptors.
///
/// Follows the HTML candidate grammar: a URL runs to the next whitespace, so commas inside URLs
/// (common on image CDNs) are preserved. Returns `None` when no candidate changed.
fn resolve_srcset(value: &str, base: &Url) -> Option<String> {
    let mut output = String::with_capacity(value.len() + 64);
    let mut changed = false;
    let mut rest = value;

    loop {
        rest = rest.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
        if rest.is_empty() {
            break;
        }
        let url_end = rest
            .find(|c: char| c.is_ascii_whitespace())
            .unwrap_or(rest.len());
        let raw_url = &rest[..url_end];
        let mut after_url = &rest[url_end..];

        // A URL ending in commas is a candidate without descriptors.
        let url = raw_url.trim_end_matches(',');
        let descriptor = if url.len() == raw_url.len() {
            let descriptor_end = after_url.find(',').unwrap_or(after_url.len());
            let descriptor = after_url[..descriptor_end].trim();
            after_url = &after_url[descriptor_end..];
            descriptor
        } else {
            ""
        };

        if !output.is_empty() {
            output.push_str(", ");
        }
        match resolve_url(url, base) {
            Some(resolved) => {
                output.push_str(&resolved);
                changed = true;
            }
            None => output.push_str(url),
        }
        if !descriptor.is_empty() {
            output.push(' ');
            output.push_str(descriptor);
        }

        assert!(after_url.len() < rest.len());
        rest = after_url;
    }

    changed.then_some(output)
}

/// Counts the characters of `html` that are outside tags.
///
/// A cheap proxy for "how much readable text does this body have", used to decide whether a feed
/// only shipped an excerpt. It does not decode entities or skip `<script>` bodies.
pub fn visible_text_length(html: &str) -> usize {
    let mut inside_tag = false;
    let mut count = 0;
    for character in html.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag && !character.is_whitespace() => count += 1,
            _ => {}
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(url: &str) -> Url {
        Url::parse(url).expect("test base URL must parse")
    }

    #[test]
    fn resolve_root_relative_src() {
        let html = r#"<img src="/images/photo.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/blog/post"));
        assert_eq!(
            result,
            r#"<img src="https://example.com/images/photo.jpg">"#
        );
    }

    #[test]
    fn resolve_path_relative_src() {
        let html = r#"<img src="images/photo.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/blog/post"));
        assert_eq!(
            result,
            r#"<img src="https://example.com/blog/images/photo.jpg">"#
        );
    }

    #[test]
    fn resolve_protocol_relative_src() {
        let html = r#"<img src="//cdn.example.com/a.png">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(result, r#"<img src="https://cdn.example.com/a.png">"#);
    }

    #[test]
    fn leave_absolute_urls_unchanged() {
        let html = r#"<img src="https://cdn.example.com/photo.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/blog"));
        assert_eq!(result, html);
    }

    #[test]
    fn skip_special_urls() {
        let html = concat!(
            r#"<img src="data:image/png;base64,abc123">"#,
            r##"<a href="#section">a</a>"##,
            r#"<a href="mailto:me@example.com">b</a>"#,
            r#"<a href="JavaScript:void(0)">c</a>"#,
            r#"<a href="tel:+1">d</a>"#,
        );
        let result = resolve_relative_urls(html, &base("https://example.com/page"));
        assert_eq!(result, html);
    }

    #[test]
    fn resolve_href_and_src_together() {
        let html = r#"<a href="/page"><img src="/img/pic.png"></a>"#;
        let result = resolve_relative_urls(html, &base("https://example.com/blog/post"));
        assert_eq!(
            result,
            r#"<a href="https://example.com/page"><img src="https://example.com/img/pic.png"></a>"#
        );
    }

    #[test]
    fn resolve_single_quoted_and_unquoted_attributes() {
        let html = "<img src='/a.jpg'><img src=/b.jpg alt=x>";
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(
            result,
            "<img src='https://example.com/a.jpg'><img src=https://example.com/b.jpg alt=x>"
        );
    }

    #[test]
    fn resolve_with_whitespace_around_equals_and_uppercase_names() {
        let html = r#"<IMG SRC = "/a.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(result, r#"<IMG SRC = "https://example.com/a.jpg">"#);
    }

    #[test]
    fn data_src_is_resolved_as_its_own_attribute() {
        let html = r#"<img data-src="/real.jpg" src="data:image/gif;base64,R0lG">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(
            result,
            r#"<img data-src="https://example.com/real.jpg" src="data:image/gif;base64,R0lG">"#
        );
    }

    #[test]
    fn unrelated_attribute_ending_in_src_is_untouched() {
        let html = r#"<div xsrc="/x" title="src=/y"></div>"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(result, html);
    }

    #[test]
    fn resolve_srcset_candidates_with_descriptors() {
        let html = r#"<img srcset="/a.jpg 1x, b.jpg 2x,/c.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/blog/post"));
        assert_eq!(
            result,
            r#"<img srcset="https://example.com/a.jpg 1x, https://example.com/blog/b.jpg 2x, https://example.com/c.jpg">"#
        );
    }

    #[test]
    fn srcset_keeps_commas_inside_urls() {
        let html = r#"<img srcset="/img/w_300,h_200/a.jpg 300w">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(
            result,
            r#"<img srcset="https://example.com/img/w_300,h_200/a.jpg 300w">"#
        );
    }

    #[test]
    fn comments_are_left_alone() {
        let html = r#"<!-- <img src="/a.jpg"> --><img src="/b.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(
            result,
            r#"<!-- <img src="/a.jpg"> --><img src="https://example.com/b.jpg">"#
        );
    }

    #[test]
    fn text_with_angle_brackets_is_not_a_tag() {
        let html = r#"1 < 2 and <img src="/a.jpg">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(result, r#"1 < 2 and <img src="https://example.com/a.jpg">"#);
    }

    #[test]
    fn malformed_html_does_not_panic() {
        let base = base("https://example.com/post");
        for html in [
            "",
            "<",
            "<img",
            "<img src=",
            r#"<img src="/unterminated"#,
            "<img =x>",
            "<!-- unterminated",
            "<a href",
        ] {
            let _ = resolve_relative_urls(html, &base);
        }
    }

    #[test]
    fn non_ascii_text_is_preserved() {
        let html = r#"<p>Zażółć</p><img src="ü.png">"#;
        let result = resolve_relative_urls(html, &base("https://example.com/post"));
        assert_eq!(
            result,
            r#"<p>Zażółć</p><img src="https://example.com/%C3%BC.png">"#
        );
    }

    #[test]
    fn visible_text_length_ignores_tags_and_whitespace() {
        assert_eq!(visible_text_length(""), 0);
        assert_eq!(visible_text_length("<p>ab c</p>"), 3);
        assert_eq!(visible_text_length(r#"<img src="x">"#), 0);
    }
}
