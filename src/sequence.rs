// Structural validation of emoji codepoint sequences.
//
// This does not aim to reproduce the full Unicode emoji-data.txt tables
// (that's several thousand entries). It hardcodes the ranges and the
// modifier-base set that cover the common cases, and is meant to grow
// as gaps are found. See README for the known limitations.

pub const ZWJ: u32 = 0x200D;
pub const VS15: u32 = 0xFE0E;
pub const VS16: u32 = 0xFE0F;
pub const KEYCAP: u32 = 0x20E3;
pub const BLACK_FLAG: u32 = 0x1F3F4;
pub const CANCEL_TAG: u32 = 0xE007F;

const EMOJI_RANGES: &[(u32, u32)] = &[
    (0x1F300, 0x1F5FF), // misc symbols and pictographs
    (0x1F600, 0x1F64F), // emoticons
    (0x1F680, 0x1F6FF), // transport and map symbols
    (0x1F7E0, 0x1F7EB), // geometric shapes extended (colored circles/squares)
    (0x1F900, 0x1F9FF), // supplemental symbols and pictographs
    (0x1FA70, 0x1FAFF), // symbols and pictographs extended-A
    (0x2600, 0x26FF),   // misc symbols
    (0x2700, 0x27BF),   // dingbats
];

// Scattered single codepoints that render as emoji but don't fall inside
// one of the ranges above.
const EMOJI_SINGLES: &[u32] = &[
    0x00A9, 0x00AE, 0x203C, 0x2049, 0x2122, 0x2139, 0x2194, 0x2195, 0x2196,
    0x2197, 0x2198, 0x2199, 0x21A9, 0x21AA, 0x231A, 0x231B, 0x2328, 0x23CF,
    0x23E9, 0x23EA, 0x23EB, 0x23EC, 0x23ED, 0x23EE, 0x23EF, 0x23F0, 0x23F1,
    0x23F2, 0x23F3, 0x23F8, 0x23F9, 0x23FA, 0x24C2, 0x25AA, 0x25AB, 0x25B6,
    0x25C0, 0x25FB, 0x25FC, 0x25FD, 0x25FE, 0x2934, 0x2935, 0x2B05, 0x2B06,
    0x2B07, 0x2B1B, 0x2B1C, 0x2B50, 0x2B55, 0x3030, 0x303D, 0x3297, 0x3299,
    0x1F004, 0x1F0CF, 0x2764, 0x2763, 0x2665,
];

// Emoji that are allowed to carry a skin tone modifier, i.e. the
// Emoji_Modifier_Base set from Unicode's emoji-data.txt. Covers the hand
// gestures, people, and body parts registered as of Unicode 15; still not
// the full table (see README).
const MODIFIER_BASE_RANGES: &[(u32, u32)] = &[
    (0x1F3C2, 0x1F3C4),
    (0x1F3CA, 0x1F3CC),
    (0x1F442, 0x1F443),
    (0x1F446, 0x1F450),
    (0x1F466, 0x1F469),
    (0x1F470, 0x1F478),
    (0x1F481, 0x1F483),
    (0x1F485, 0x1F487),
    (0x1F574, 0x1F575),
    (0x1F595, 0x1F596),
    (0x1F645, 0x1F647),
    (0x1F64B, 0x1F64F),
    (0x1F6B4, 0x1F6B6),
    (0x1F918, 0x1F91F),
    (0x1F937, 0x1F939),
    (0x1F9B5, 0x1F9B6),
    (0x1F9B8, 0x1F9B9),
    (0x1F9CD, 0x1F9CF),
    (0x1F9D1, 0x1F9DD),
];

const MODIFIER_BASE_SINGLES: &[u32] = &[
    0x261D, 0x26F9, 0x270A, 0x270B, 0x270C, 0x270D, 0x1F385, 0x1F3C7,
    0x1F47C, 0x1F48F, 0x1F491, 0x1F4AA, 0x1F57A, 0x1F590, 0x1F6A3, 0x1F6C0,
    0x1F6CC, 0x1F90C, 0x1F90F, 0x1F926, 0x1F930, 0x1F93D, 0x1F93E, 0x1F977,
    0x1F9BB,
];

pub fn is_emoji_scalar(cp: u32) -> bool {
    EMOJI_RANGES.iter().any(|&(lo, hi)| cp >= lo && cp <= hi) || EMOJI_SINGLES.contains(&cp)
}

pub fn is_modifier_base(cp: u32) -> bool {
    MODIFIER_BASE_RANGES.iter().any(|&(lo, hi)| cp >= lo && cp <= hi)
        || MODIFIER_BASE_SINGLES.contains(&cp)
}

pub fn is_regional_indicator(cp: u32) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&cp)
}

pub fn is_skin_tone_modifier(cp: u32) -> bool {
    (0x1F3FB..=0x1F3FF).contains(&cp)
}

pub fn is_tag_char(cp: u32) -> bool {
    (0xE0020..=0xE007E).contains(&cp)
}

fn is_keycap_base(cp: u32) -> bool {
    (b'0' as u32..=b'9' as u32).contains(&cp) || cp == b'#' as u32 || cp == b'*' as u32
}

fn is_known_codepoint(cp: u32) -> bool {
    is_emoji_scalar(cp)
        || is_regional_indicator(cp)
        || is_skin_tone_modifier(cp)
        || is_tag_char(cp)
        || is_keycap_base(cp)
        || matches!(cp, ZWJ | VS15 | VS16 | KEYCAP | BLACK_FLAG | CANCEL_TAG)
}

/// Validate a sequence of codepoints. `lenient` relaxes checks that are
/// about "does this sequence make sense" (dangling joiners, wrong base for
/// a modifier, mixed flag components) but never lets through a codepoint
/// that has nothing to do with emoji at all.
pub fn validate(cps: &[u32], lenient: bool) -> Result<(), String> {
    if cps.is_empty() {
        return Err("empty sequence".to_string());
    }

    for (i, &cp) in cps.iter().enumerate() {
        if !is_known_codepoint(cp) {
            return Err(format!(
                "U+{:04X} at position {} is not a recognized emoji-related codepoint",
                cp, i
            ));
        }
    }

    // A lone black flag (or one followed by a joiner) is just the ordinary
    // "waving black flag" emoji; it's only a tag sequence once a tag
    // character follows it.
    if cps[0] == BLACK_FLAG && cps.len() > 1 && is_tag_char(cps[1]) {
        return validate_tag_sequence(cps, lenient);
    }

    if cps.iter().any(|&c| is_regional_indicator(c)) {
        return validate_flag_sequence(cps, lenient);
    }

    if cps.iter().any(|&c| c == KEYCAP) {
        return validate_keycap_sequence(cps, lenient);
    }

    validate_general_sequence(cps, lenient)
}

fn validate_flag_sequence(cps: &[u32], lenient: bool) -> Result<(), String> {
    if !cps.iter().all(|&c| is_regional_indicator(c)) {
        return Err("regional indicators cannot be combined with other emoji".to_string());
    }
    if cps.len() % 2 != 0 {
        return Err("regional indicators must come in pairs".to_string());
    }
    if !lenient && cps.len() != 2 {
        return Err(format!(
            "flag sequence has {} regional indicators, expected exactly 2 (use --lenient to allow subdivision-style runs)",
            cps.len()
        ));
    }
    Ok(())
}

fn validate_keycap_sequence(cps: &[u32], lenient: bool) -> Result<(), String> {
    match cps {
        [base, vs, kc] if is_keycap_base(*base) && *vs == VS16 && *kc == KEYCAP => Ok(()),
        [base, kc] if lenient && is_keycap_base(*base) && *kc == KEYCAP => Ok(()),
        _ => Err("keycap sequence must be a digit, '#' or '*' followed by U+FE0F and U+20E3".to_string()),
    }
}

fn validate_tag_sequence(cps: &[u32], lenient: bool) -> Result<(), String> {
    if cps.len() < 2 {
        return Err("tag sequence needs at least one tag character after the black flag".to_string());
    }
    let body_end = if *cps.last().unwrap() == CANCEL_TAG {
        cps.len() - 1
    } else if lenient {
        cps.len()
    } else {
        return Err("tag sequence must end with the cancel tag U+E007F".to_string());
    };
    if body_end <= 1 {
        return Err("tag sequence has no tag characters".to_string());
    }
    if !cps[1..body_end].iter().all(|&c| is_tag_char(c)) {
        return Err("tag sequence body must consist only of tag characters".to_string());
    }
    Ok(())
}

fn validate_general_sequence(cps: &[u32], lenient: bool) -> Result<(), String> {
    if !is_emoji_scalar(cps[0]) {
        return Err(format!(
            "sequence cannot start with U+{:04X}; it must start with a base emoji",
            cps[0]
        ));
    }

    for i in 1..cps.len() {
        let prev = cps[i - 1];
        let cur = cps[i];

        if cur == ZWJ {
            if i == cps.len() - 1 {
                return Err("sequence cannot end with a joiner".to_string());
            }
            if prev == ZWJ {
                return Err(format!("two joiners in a row at position {}", i));
            }
            continue;
        }

        if cur == VS15 || cur == VS16 {
            if prev == ZWJ {
                return Err(format!(
                    "variation selector at position {} must follow an emoji, not a joiner",
                    i
                ));
            }
            continue;
        }

        if is_skin_tone_modifier(cur) {
            if prev == ZWJ {
                return Err(format!(
                    "skin tone modifier at position {} cannot immediately follow a joiner",
                    i
                ));
            }
            if is_skin_tone_modifier(prev) {
                return Err(format!("two skin tone modifiers in a row at position {}", i));
            }
            if !lenient && !is_modifier_base(prev) {
                return Err(format!(
                    "skin tone modifier at position {} is applied to U+{:04X}, which is not a modifier base",
                    i, prev
                ));
            }
            continue;
        }

        if is_emoji_scalar(cur) {
            if prev != ZWJ && !lenient {
                return Err(format!(
                    "U+{:04X} at position {} follows another emoji without a joiner between them",
                    cur, i
                ));
            }
            continue;
        }

        return Err(format!("unexpected codepoint U+{:04X} at position {}", cur, i));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const WAVING_HAND: u32 = 0x1F44B;
    const WOMAN: u32 = 0x1F469;
    const LAPTOP: u32 = 0x1F4BB;
    const THUMBS_UP: u32 = 0x1F44D;
    const HEART: u32 = 0x2764;
    const REGIONAL_U: u32 = 0x1F1FA;
    const REGIONAL_S: u32 = 0x1F1F8;

    #[test]
    fn rejects_empty_sequence() {
        assert!(validate(&[], false).is_err());
    }

    #[test]
    fn rejects_unrelated_codepoint() {
        let err = validate(&[b'A' as u32], false).unwrap_err();
        assert!(err.contains("not a recognized"));
    }

    #[test]
    fn accepts_single_emoji() {
        assert!(validate(&[WAVING_HAND], false).is_ok());
    }

    #[test]
    fn accepts_zwj_sequence() {
        assert!(validate(&[WOMAN, ZWJ, LAPTOP], false).is_ok());
    }

    #[test]
    fn rejects_leading_joiner() {
        let err = validate(&[ZWJ, WAVING_HAND], false).unwrap_err();
        assert!(err.contains("cannot start"));
    }

    #[test]
    fn rejects_trailing_joiner() {
        let err = validate(&[WAVING_HAND, ZWJ], false).unwrap_err();
        assert!(err.contains("cannot end with a joiner"));
    }

    #[test]
    fn rejects_doubled_joiner() {
        let err = validate(&[WOMAN, ZWJ, ZWJ, LAPTOP], false).unwrap_err();
        assert!(err.contains("two joiners in a row"));
    }

    #[test]
    fn strict_rejects_skin_tone_on_non_modifier_base() {
        let err = validate(&[HEART, 0x1F3FB], false).unwrap_err();
        assert!(err.contains("not a modifier base"));
    }

    #[test]
    fn lenient_allows_skin_tone_on_non_modifier_base() {
        assert!(validate(&[HEART, 0x1F3FB], true).is_ok());
    }

    #[test]
    fn accepts_skin_tone_on_modifier_base() {
        assert!(validate(&[THUMBS_UP, 0x1F3FB], false).is_ok());
    }

    #[test]
    fn rejects_doubled_skin_tone_modifier() {
        let err = validate(&[THUMBS_UP, 0x1F3FB, 0x1F3FC], false).unwrap_err();
        assert!(err.contains("two skin tone modifiers"));
    }

    #[test]
    fn rejects_skin_tone_modifier_after_joiner() {
        let err = validate(&[WOMAN, ZWJ, 0x1F3FB], false).unwrap_err();
        assert!(err.contains("cannot immediately follow a joiner"));
    }

    #[test]
    fn accepts_two_letter_flag() {
        assert!(validate(&[REGIONAL_U, REGIONAL_S], false).is_ok());
    }

    #[test]
    fn strict_rejects_odd_regional_indicator_count() {
        let err = validate(&[REGIONAL_U, REGIONAL_S, REGIONAL_U], false).unwrap_err();
        assert!(err.contains("must come in pairs"));
    }

    #[test]
    fn strict_rejects_more_than_two_regional_indicators() {
        let err = validate(&[REGIONAL_U, REGIONAL_S, REGIONAL_U, REGIONAL_S], false).unwrap_err();
        assert!(err.contains("expected exactly 2"));
    }

    #[test]
    fn lenient_allows_regional_indicator_run() {
        assert!(validate(&[REGIONAL_U, REGIONAL_S, REGIONAL_U, REGIONAL_S], true).is_ok());
    }

    #[test]
    fn rejects_regional_indicator_mixed_with_other_emoji() {
        let err = validate(&[REGIONAL_U, WAVING_HAND], false).unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn accepts_full_keycap_sequence() {
        assert!(validate(&[b'1' as u32, VS16, KEYCAP], false).is_ok());
    }

    #[test]
    fn strict_rejects_keycap_missing_variation_selector() {
        assert!(validate(&[b'1' as u32, KEYCAP], false).is_err());
    }

    #[test]
    fn lenient_allows_keycap_missing_variation_selector() {
        assert!(validate(&[b'1' as u32, KEYCAP], true).is_ok());
    }

    #[test]
    fn accepts_full_tag_sequence() {
        // "GB" flag tag sequence, black flag + tag letters g,b + cancel tag.
        let seq = [
            BLACK_FLAG,
            0xE0067,
            0xE0062,
            CANCEL_TAG,
        ];
        assert!(validate(&seq, false).is_ok());
    }

    #[test]
    fn strict_rejects_tag_sequence_without_cancel_tag() {
        let seq = [BLACK_FLAG, 0xE0067, 0xE0062];
        assert!(validate(&seq, false).is_err());
    }

    #[test]
    fn lenient_allows_tag_sequence_without_cancel_tag() {
        let seq = [BLACK_FLAG, 0xE0067, 0xE0062];
        assert!(validate(&seq, true).is_ok());
    }

    #[test]
    fn accepts_lone_black_flag_as_ordinary_emoji() {
        assert!(validate(&[BLACK_FLAG], false).is_ok());
    }

    #[test]
    fn recognizes_colored_circle_range() {
        // large orange circle
        assert!(validate(&[0x1F7E0], false).is_ok());
    }

    #[test]
    fn recognizes_arrow_and_shape_singles() {
        assert!(is_emoji_scalar(0x2B50)); // star
        assert!(is_emoji_scalar(0x00A9)); // copyright
    }

    #[test]
    fn accepts_skin_tone_on_newly_added_modifier_bases() {
        // person golfing, sign of the horns, standing person
        assert!(validate(&[0x1F3CC, 0x1F3FB], false).is_ok());
        assert!(validate(&[0x1F918, 0x1F3FC], false).is_ok());
        assert!(validate(&[0x1F9CD, 0x1F3FD], false).is_ok());
    }

    #[test]
    fn strict_rejects_skin_tone_on_non_modifier_person_variant() {
        // zombie: person-adjacent but not a modifier base
        let err = validate(&[0x1F9DF, 0x1F3FB], false).unwrap_err();
        assert!(err.contains("not a modifier base"));
    }
}
