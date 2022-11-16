use std::ops::RangeInclusive;
use std::str::FromStr;

use std::char;

const HIGH_SURROGATES: RangeInclusive<u32> = 0xD800..=0xDBFF;
const LOW_SURROGATES: RangeInclusive<u32> = 0xDC00..=0xDFFF;

fn decode_utf16_surrogates(low: u32, high: u32) -> char {
    assert!(LOW_SURROGATES.contains(&low));
    assert!(HIGH_SURROGATES.contains(&high));
    let value = 0x010000 + (((high - 0xD800) << 10) | (low - 0xDC00));
    let c = char::from_u32(value).unwrap();
    let mut buf = [0u16; 2];
    debug_assert_eq!(
        *c.encode_utf16(&mut buf),
        [high as u16, low as u16],
        "Unexpected roundtrip for {:?}",
        c
    );
    c
}

/// Replaces HTML escaped utf16 surrogates with their correct counterparts
pub fn cleanup_html_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while !remaining.is_empty() {
        if remaining.starts_with("&#") {
            let end = remaining.find(';').unwrap();
            let escape = &remaining[..=end];
            let escape_number = parse_escape_number(escape);
            remaining = &remaining[end + 1..];
            if HIGH_SURROGATES.contains(&escape_number) {
                assert!(
                    remaining.starts_with("&#"),
                    "Invalid remaining after surrogate: {}",
                    &remaining[..5]
                );
                let low_end = remaining.find(';').unwrap();
                let low_escape = &remaining[..=low_end];
                let low_number = parse_escape_number(low_escape);
                remaining = &remaining[(low_end + 1)..];
                let c = decode_utf16_surrogates(low_number, escape_number);
                result.push_str(&format!("&#x{:X};", c as u32))
            } else {
                result.push_str(escape)
            }
        } else {
            let c = remaining.chars().next().unwrap();
            result.push(c);
            remaining = &remaining[c.len_utf8()..];
        }
    }
    result
}

fn parse_escape_number(escape: &str) -> u32 {
    u32::from_str(&escape[2..escape.len() - 1])
        .unwrap_or_else(|_| panic!("Invalid numeric escape: {:?}", escape))
}

#[cfg(test)]
mod test {
    use super::cleanup_html_escapes;
    #[test]
    fn test_basic() {
        assert_eq!(
            cleanup_html_escapes("??? Whoop whoop! &#55357;&#56842; asdf"),
            "??? Whoop whoop! &#x1F60A; asdf"
        );
        assert_eq!(
            cleanup_html_escapes(
                "Same to you! &#55356;&#57222;&#55356;&#56826;&#55356;&#56818; asdf-testing;;"
            ),
            "Same to you! &#x1F386;&#x1F1FA;&#x1F1F2; asdf-testing;;"
        )
    }
}
