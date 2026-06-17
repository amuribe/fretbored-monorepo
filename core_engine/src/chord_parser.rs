//! Parses chord symbols and separate-arg chord commands into `(Note, ChordType)`.
//!
//! Supports two input forms:
//!   1. Single symbol:      "Cm7", "C#maj9", "Ebsus4", "F#dim7"
//!   2. Separate arguments:  root="G" quality="minor" extension=Some("9")
//!
//! Grammar (single-symbol form):
//!   <root letter><optional accidental><quality token><optional accidental><extension digits>
//!
//! Quality tokens (prefix-matched, case rules noted):
//!   "m"  (lowercase only) / "min" / "minor"        -> Minor
//!   "M"  (uppercase only) / "maj" / "major"        -> Major
//!   "dim" / "diminished"                            -> Diminished
//!   "aug" / "augmented"                              -> Augmented
//!   "sus2"                                           -> Sus2
//!   "sus4"                                           -> Sus4
//!   (none)                                         -> Major (default)

use crate::theory::Note;
use crate::theory::chord::{ChordQuality, ChordType, Extension};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ChordParseError(pub String);

impl std::fmt::Display for ChordParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse chord: {}", self.0)
    }
}

impl std::error::Error for ChordParseError {}

/// Distinguishes an explicitly-typed quality from a defaulted one
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum QualityToken {
    Minor,
    Major,
    /// Major because no quality token was present at all (e.g. "C7", "C9").
    /// Distinct from an explicitly-typed "M"/"maj"/"major" so the extension
    /// parser can tell them apart
    DefaultMajor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
}

/// Parse a root note letter + optional accidental from the start of `chord_str`.
/// Returns (Note(e.g. C, Bb, F#), rest(e.g. "m7", "maj9", ""<empty>))
pub fn parse_root(chord_str: &str) -> Result<(Note, &str), ChordParseError> {
    // Convert string slice into a byte slice for O(1) lookups
    let bytes = chord_str.as_bytes();

    if bytes.is_empty() {
        return Err(ChordParseError("empty chord symbol".to_string()));
    }

    // The root note letter (e.g. C, D) accidental is checked after
    let letter = bytes[0];

    if !letter.is_ascii_alphabetic() {
        return Err(ChordParseError(format!(
            "expected an alphabetical note letter, got '{}'",
            chord_str
        )));
    }

    // Check for accidental should be immediately after the letter
    // No Accidental: `root_str` len = 1, `rest` = 1..
    // Accidental: `root_str` len = 2, `rest` = 2..
    let (root_str, rest) = if bytes.len() > 1 && (bytes[1] == b'#' || bytes[1] == b'b') {
        // Contains accidental
        (&chord_str[0..2], &chord_str[2..])
    } else {
        // No accidental
        (&chord_str[0..1], &chord_str[1..])
    };

    let note = Note::from_str(root_str)
        .map_err(|e| ChordParseError(format!("invalid root note '{}': {}", root_str, e)))?;

    Ok((note, rest))
}

/// Case-insensitive prefix stripping helper function
fn strip_prefix_ignore_case<'a>(blob: &'a str, prefix: &str) -> Option<&'a str> {
    let len = prefix.len();

    blob.get(..len)?
        .eq_ignore_ascii_case(prefix)
        .then(|| &blob[len..])
}

/// Parses a chord quality token from the start of a string slice.
///
/// Case-sensitive for exact matches: `"m"` (Minor), `"M"` (Major).
/// Prefix-matched for full words: `"min"`, `"dim"`, `"sus4"`, etc.
/// Defaults to `QualityToken::DefaultMajor` (consuming 0 bytes) if no match is found.
///
/// # Special Case: The "maj" Prefix
/// Standard parsing would strip `"maj"` and leave the trailing digits (e.g., `"maj7"` -> `"7"`).
/// Passing a bare `"7"` to the extension parser incorrectly generates a Dominant 7th instead of a Major 7th.
///
/// To prevent this, if `"maj"` is immediately followed by a digit:
/// The function returns `QualityToken::Major` (explicit, not defaulted),
/// then the full string (e.g., `"maj7"`) is passed in full to `parse_extension`.
fn parse_quality(blob: &str) -> (QualityToken, &str) {
    // Prevent splitting word form extension into a quality
    if let Some(rest) = blob.strip_prefix("maj")
        && rest
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        // Leave "majN" string intact as extension token
        return (QualityToken::Major, blob);
    }

    // Check full words (case-insensitive) first to avoid matching "m" in "maj" as "m" for "minor"
    if let Some(rest) = strip_prefix_ignore_case(blob, "sus2") {
        return (QualityToken::Sus2, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "sus4") {
        return (QualityToken::Sus4, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "diminished") {
        return (QualityToken::Diminished, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "dim") {
        return (QualityToken::Diminished, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "augmented") {
        return (QualityToken::Augmented, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "aug") {
        return (QualityToken::Augmented, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "minor") {
        return (QualityToken::Minor, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "min") {
        return (QualityToken::Minor, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "major") {
        return (QualityToken::Major, rest);
    }
    if let Some(rest) = strip_prefix_ignore_case(blob, "maj") {
        return (QualityToken::Major, rest);
    }

    // Case-sensitive single letters or symbols ('m' -> Minor, '+' -> Augmented)
    if let Some(rest) = blob.strip_prefix('m') {
        return (QualityToken::Minor, rest);
    }

    if let Some(rest) = blob.strip_prefix('-') {
        return (QualityToken::Minor, rest);
    }

    if let Some(rest) = blob.strip_prefix('M') {
        return (QualityToken::Major, rest);
    }

    if let Some(rest) = blob.strip_prefix('+') {
        return (QualityToken::Augmented, rest);
    }

    if let Some(rest) = blob.strip_prefix('o') {
        return (QualityToken::Diminished, rest);
    }

    // No quality token recognized -> defaulted Major
    // Kept distinct from an explicitly-typed Major
    (QualityToken::DefaultMajor, blob)
}

/// Parse the extension (after quality has been stripped)
fn parse_extension(ext_str: &str, quality: QualityToken) -> Result<Extension, ChordParseError> {
    let trimmed = ext_str.trim();
    // If `ext_str` is empty, extension is None
    if trimmed.is_empty() {
        return Ok(Extension::None);
    }

    // Check word form extensions first
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "maj7" => return Ok(Extension::MajorSeventh),
        "dom7" => return Ok(Extension::DominantSeventh),
        "dim7" => return Ok(Extension::DiminishedSeventh),
        "maj9" => return Ok(Extension::MajorNinth),
        "maj11" => return Ok(Extension::MajorEleventh),
        "maj13" => return Ok(Extension::MajorThirteenth),
        "6" => return Ok(Extension::Sixth),
        "69" | "6/9" => return Ok(Extension::SixNine),
        _ => {}
    }
    // Literal "M7" (capital, case-sensitive): this is NOT reachable from a
    // bare "CM7" input, since parse_quality already strips a leading M there,
    // leaving plain "7" for the numeric path. This IS reachable from "CmM7"
    // (minor-major-7th): parse_quality strips the leading lowercase "m" as
    // Minor quality, leaving the literal string "M7" — which has no digit-only
    // form, so it must be special-cased here rather than via the numeric path.
    if trimmed == "M7" {
        return Ok(Extension::MajorSeventh);
    }

    // Numeric / altered-numeric form: optional leading b/# then digits
    let (alteration, digits) = match trimmed.as_bytes().first() {
        Some(b'b') => (Some('b'), &trimmed[1..]),
        Some(b'#') => (Some('#'), &trimmed[1..]),
        _ => (None, trimmed),
    };

    let number: u32 = digits
        .parse()
        .map_err(|_| ChordParseError(format!("invalid extension '{}'", ext_str)))?;

    use QualityToken::*;
    let ext = match (number, alteration, quality) {
        (7, None, Diminished) => Extension::DiminishedSeventh,
        (7, None, Minor) => Extension::MinorSeventh,
        (7, None, Augmented) => Extension::DominantSeventh, // 7#5, AugmentedSeventh
        // Explicit Major ("M7") means Cmaj7-equivalent: MajorSeventh.
        // Defaulted Major (no quality token, e.g. "C7") is dominant: DominantSeventh.
        (7, None, Major) => Extension::MajorSeventh,
        (7, None, DefaultMajor) => Extension::DominantSeventh,
        (7, None, Sus2 | Sus4) => Extension::DominantSeventh,

        // Bare "9" is dominant regardless of explicit vs defaulted Major
        (9, None, Major) => Extension::MajorNinth,
        (9, None, _) => Extension::Ninth,
        (9, Some('b'), _) => Extension::FlatNinth,
        (9, Some('#'), _) => Extension::SharpNinth,

        // 11/13: explicit Major (typed M/maj/major) upgrades to the Major variant;
        // defaulted Major (no token at all e.g. "C11") stays dominant, same as 9.
        (11, None, Major) => Extension::MajorEleventh,
        (11, None, _) => Extension::Eleventh,
        (11, Some('#'), _) => Extension::SharpEleventh,

        (13, None, Major) => Extension::MajorThirteenth,
        (13, None, _) => Extension::Thirteenth,
        (13, Some('b'), _) => Extension::FlatThirteenth,

        _ => {
            return Err(ChordParseError(format!(
                "unsupported extension '{}' for given quality",
                ext_str
            )));
        }
    };

    Ok(ext)
}

fn to_chord_quality(token: QualityToken) -> ChordQuality {
    match token {
        QualityToken::Minor => ChordQuality::Minor,
        QualityToken::Major | QualityToken::DefaultMajor => ChordQuality::Major,
        QualityToken::Diminished => ChordQuality::Diminished,
        QualityToken::Augmented => ChordQuality::Augmented,
        QualityToken::Sus2 => ChordQuality::Sus2,
        QualityToken::Sus4 => ChordQuality::Sus4,
    }
}

/// Parse the separate-arg form: root="G", quality="minor", extension=Some("9").
pub fn parse_chord_parts(
    root_str: &str,
    quality_str: &str,
    extension_str: Option<&str>,
) -> Result<(Note, ChordType), ChordParseError> {
    let root = Note::from_str(root_str)
        .map_err(|e| ChordParseError(format!("invalid root note '{}': {}", root_str, e)))?;

    // Reuse the same quality-token matcher by feeding it the quality arg
    // directly (it should fully consume it; leftover means an unrecognized word).
    let (quality_token, leftover) = parse_quality(quality_str);
    if !leftover.is_empty() {
        return Err(ChordParseError(format!(
            "unrecognized quality '{}'",
            quality_str
        )));
    }

    let extension = match extension_str {
        Some(e) => parse_extension(e, quality_token)?,
        None => Extension::None,
    };

    Ok((
        root,
        ChordType {
            quality: to_chord_quality(quality_token),
            extension,
        },
    ))
}

/// Parse a single chord symbol string (e.g., "Cm7", "F#maj9").
pub fn parse_chord_symbol(symbol: &str) -> Result<(Note, ChordType), ChordParseError> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err(ChordParseError("Empty chord symbol".to_string()));
    }

    // Isolate root and remainder
    let (root, rest1) = parse_root(trimmed)?;

    // Isolate quality and remainder
    let (quality_token, rest2) = parse_quality(rest1);

    // Parse remaining digits/tokens as extension
    let extension = parse_extension(rest2, quality_token)?;

    Ok((
        root,
        ChordType {
            quality: to_chord_quality(quality_token),
            extension,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::chord::{ChordQuality, Extension};

    #[test]
    fn test_plain_major_triad() {
        let (root, ct) = parse_chord_symbol("C").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::None);
    }

    #[test]
    fn test_minor_seventh() {
        let (root, ct) = parse_chord_symbol("Cm7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Minor);
        assert_eq!(ct.extension, Extension::MinorSeventh);
    }

    #[test]
    fn test_dominant_seventh_default_major() {
        let (root, ct) = parse_chord_symbol("C7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::DominantSeventh);
    }

    #[test]
    fn test_explicit_major_seventh() {
        let (root, ct) = parse_chord_symbol("CM7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::MajorSeventh);
    }

    #[test]
    fn test_maj7_word_form() {
        let (root, ct) = parse_chord_symbol("Cmaj7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::MajorSeventh);
    }

    #[test]
    fn test_minor_major_seventh() {
        // CmM7 -> minor triad + major 7th interval
        let (root, ct) = parse_chord_symbol("CmM7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Minor);
        assert_eq!(ct.extension, Extension::MajorSeventh);
    }

    #[test]
    fn test_augmented_seven_sharp_five() {
        // Caug7 -> Augmented quality (M3 + #5) + DominantSeventh (b7)
        let (root, ct) = parse_chord_symbol("Caug7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Augmented);
        assert_eq!(ct.extension, Extension::DominantSeventh);
    }

    #[test]
    fn test_diminished_seventh() {
        let (root, ct) = parse_chord_symbol("Cdim7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Diminished);
        assert_eq!(ct.extension, Extension::DiminishedSeventh);
    }

    #[test]
    fn test_sharp_root() {
        let (root, ct) = parse_chord_symbol("C#m7").unwrap();
        assert_eq!(root, Note::CSharp);
        assert_eq!(ct.quality, ChordQuality::Minor);
        assert_eq!(ct.extension, Extension::MinorSeventh);
    }

    #[test]
    fn test_flat_root_maj9() {
        let (root, ct) = parse_chord_symbol("Ebmaj9").unwrap();
        assert_eq!(root, Note::DSharp);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::MajorNinth);
    }

    #[test]
    fn test_sus4_stacks_extension_normally() {
        let (root, ct) = parse_chord_symbol("Csus4").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct.quality, ChordQuality::Sus4);
        assert_eq!(ct.extension, Extension::None);
    }

    #[test]
    fn test_root_accidental_consumes_the_only_sign_slot() {
        // The root's accidental (#/b) is the ONLY accidental slot in the grammar.
        // Once consumed by the root, there is no sign left for the extension,
        // so "C#9" and "Db9" are root=C#/Db(=C#) + plain Ninth, NOT SharpNinth/FlatNinth.
        let (root_sharp, ct_sharp) = parse_chord_symbol("C#9").unwrap();
        assert_eq!(root_sharp, Note::CSharp);
        assert_eq!(ct_sharp.extension, Extension::Ninth);

        let (root_flat, ct_flat) = parse_chord_symbol("Db9").unwrap();
        assert_eq!(root_flat, Note::CSharp); // Db is enharmonic C#
        assert_eq!(ct_flat.extension, Extension::Ninth);
    }

    #[test]
    fn test_altered_extension_requires_explicit_major_to_disambiguate() {
        // To get an altered 9th/11th/13th on a natural-letter root (no quality
        // token), the grammar requires an explicit "M" first so the trailing
        // b/# is unambiguously the extension's sign, not a second root accidental.
        let (root, ct_flat9) = parse_chord_symbol("CMb9").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(ct_flat9.extension, Extension::FlatNinth);

        let (_, ct_sharp11) = parse_chord_symbol("CM#11").unwrap();
        assert_eq!(ct_sharp11.extension, Extension::SharpEleventh);
    }

    #[test]
    fn test_sharp_eleventh() {
        // Without an explicit M, "#11" right after the root letter is read as
        // the root's accidental, leaving plain Eleventh (see test above for
        // the disambiguated form).
        let (root, ct) = parse_chord_symbol("C#11").unwrap();
        assert_eq!(root, Note::CSharp);
        assert_eq!(ct.extension, Extension::Eleventh);
    }

    #[test]
    fn test_explicit_major_eleventh_and_thirteenth() {
        // "M" is explicit Major -> bare 11/13 upgrade to Major variants
        let (_, ct11) = parse_chord_symbol("CM11").unwrap();
        assert_eq!(ct11.extension, Extension::MajorEleventh);

        let (_, ct13) = parse_chord_symbol("CM13").unwrap();
        assert_eq!(ct13.extension, Extension::MajorThirteenth);
    }

    #[test]
    fn test_defaulted_major_eleventh_and_thirteenth_stay_plain() {
        // No quality token at all -> defaulted Major -> plain (dominant-flavored) variants
        let (_, ct11) = parse_chord_symbol("C11").unwrap();
        assert_eq!(ct11.extension, Extension::Eleventh);

        let (_, ct13) = parse_chord_symbol("C13").unwrap();
        assert_eq!(ct13.extension, Extension::Thirteenth);
    }

    #[test]
    fn test_flat_thirteenth_via_explicit_major() {
        // Same disambiguation rule as b9/#11: bare "Ebb13" would be malformed
        // (Eb is the root, second b can't also be root's), so this uses a
        // plain-letter root for the bare case and Eb for the explicit-M case.
        let (root_bare, ct_bare) = parse_chord_symbol("Db13").unwrap();
        assert_eq!(root_bare, Note::CSharp); // Db is enharmonic C#
        assert_eq!(ct_bare.extension, Extension::Thirteenth);

        let (root, ct) = parse_chord_symbol("EbMb13").unwrap();
        assert_eq!(root, Note::DSharp); // Eb root
        assert_eq!(ct.extension, Extension::FlatThirteenth);
    }

    #[test]
    fn test_separate_arg_form() {
        let (root, ct) = parse_chord_parts("G", "minor", Some("9")).unwrap();
        assert_eq!(root, Note::G);
        assert_eq!(ct.quality, ChordQuality::Minor);
        assert_eq!(ct.extension, Extension::Ninth);
    }

    #[test]
    fn test_separate_arg_form_word_extension() {
        let (root, ct) = parse_chord_parts("G", "major", Some("maj7")).unwrap();
        assert_eq!(root, Note::G);
        assert_eq!(ct.quality, ChordQuality::Major);
        assert_eq!(ct.extension, Extension::MajorSeventh);
    }

    #[test]
    fn test_separate_arg_form_no_extension() {
        let (root, ct) = parse_chord_parts("D", "minor", None).unwrap();
        assert_eq!(root, Note::D);
        assert_eq!(ct.quality, ChordQuality::Minor);
        assert_eq!(ct.extension, Extension::None);
    }

    #[test]
    fn test_invalid_root_rejected() {
        assert!(parse_chord_symbol("H7").is_err());
    }

    #[test]
    fn test_invalid_quality_word_rejected_in_parts_form() {
        assert!(parse_chord_parts("C", "bogus", None).is_err());
    }
}
