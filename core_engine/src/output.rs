// Format a voicing as a readable String output
use std::ops::RangeInclusive;

use crate::{
    instrument::Tuning,
    voicing::{Voicing, calculate_open_count},
    voicing_ranker::{calculate_stretch, min_max_fret},
};

// Holds a voicing and metadata formatted for display.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoicingDisplay<'a> {
    pub voicing: Voicing,
    pub tuning: &'a Tuning<'a>,
    pub note_names: Vec<String>,
}

impl<'a> VoicingDisplay<'a> {
    pub fn new(voicing: Voicing, tuning: &'a Tuning<'a>, use_flats: bool) -> Self {
        let note_names = voicing
            .frets
            .iter()
            .enumerate()
            .map(|(string_idx, fret_opt)| match fret_opt {
                Some(fret) => tuning
                    .note_at_fret(string_idx, *fret)
                    .map(|n| n.name(use_flats).to_string())
                    .unwrap_or_else(|| "-".to_string()),
                None => "-".to_string(),
            })
            .collect();

        Self {
            voicing,
            tuning,
            note_names,
        }
    }
}

pub fn format_voicing(voicing: &Voicing, tuning: &Tuning, use_flats: bool) -> String {
    // Build the fret array: e.g. G [3, 2, 0, 0, 3, 3]
    // "-" = empty or muted string
    let fret_str = format!(
        "[{}]",
        voicing
            .frets
            .iter()
            .map(|f| match f {
                Some(fret) => fret.to_string(),
                None => "-".to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Get the note name at each fret, depending on tuning / instrument
    let note_names: Vec<&str> = voicing
        .frets
        .iter()
        .enumerate()
        .map(|(string_idx, fret_opt)| match fret_opt {
            Some(fret) => tuning
                .note_at_fret(string_idx, *fret)
                .map(|n| n.name(use_flats))
                .unwrap_or("X"),
            None => "-",
        })
        .collect();

    format!("Frets: {} | Notes: {}", fret_str, note_names.join(" "))
}

pub fn format_voicing_w_playability(voicing: &Voicing, tuning: &Tuning, use_flats: bool) -> String {
    // Format voicing to string with plauability information
    let base_str = format_voicing(voicing, tuning, use_flats);

    // Calculate the largest stretch in the voicing, ignoring open strings and muted strings
    let stretch = calculate_stretch(voicing);

    let open_count = calculate_open_count(voicing);
    format!("{} | Stretch: {} | Open: {}", base_str, stretch, open_count)
}

/// Helper which determines the fret range
pub fn fret_window(
    voicing: &Voicing,
    capo: u8,
    fret_count: u8,
    min_span: u8,
) -> RangeInclusive<u8> {
    let (min, max) = min_max_fret(voicing).unwrap_or((0, 0));

    let has_open = voicing.frets.contains(&Some(0));

    // Get the lower boundary of the window
    let low = if capo > 0 {
        capo - 1
    } else if has_open || min <= 2 {
        0
    } else {
        min - 1
    };

    // Ensure hi is clamped to fret count
    let hi = max
        .saturating_add(1)
        .max(low.saturating_add(min_span))
        .min(fret_count);

    // Return the range
    low..=hi
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instrument::StandardTunings, voicing::Voicing};

    #[test]
    fn test_format_voicing() {
        let voicing = Voicing {
            frets: vec![Some(0), Some(3), Some(2), None, Some(1), Some(0)],
        };
        let tuning = StandardTunings::guitar_standard();
        let formatted = format_voicing(&voicing, &tuning, false);
        assert!(formatted.contains("[0, 3, 2, -, 1, 0]"));
        assert!(formatted.contains("Frets:"));
        assert!(formatted.contains("Notes:"));
    }

    #[test]
    fn test_format_voicing_w_playability() {
        let voicing = Voicing {
            frets: vec![None, Some(3), Some(2), Some(0), Some(1), Some(0)],
        };
        let tuning = StandardTunings::guitar_standard();
        let formatted = format_voicing_w_playability(&voicing, &tuning, false);
        assert!(formatted.contains("Stretch: 2"));
        assert!(
            formatted.contains("Open: 2"),
            "Expected 'Open: 2', Actual: {}",
            formatted
        );
    }

    fn v(frets: &[Option<u8>]) -> Voicing {
        Voicing {
            frets: frets.to_vec(),
        }
    }

    #[test]
    fn window_open_chord_starts_at_nut() {
        let c = v(&[None, Some(3), Some(2), Some(0), Some(1), Some(0)]);
        assert_eq!(fret_window(&c, 0, 24, 5), 0..=5);
    }

    #[test]
    fn window_barre_chord_tracks_position() {
        // lo = 8 - 1 = 7; hi = max(10 + 1, 7 + 5) = 12
        let f = v(&[Some(8), Some(10), Some(10), Some(9), Some(8), Some(8)]);
        assert_eq!(fret_window(&f, 0, 24, 5), 7..=12);
    }

    #[test]
    fn window_capo_starts_before_capo() {
        let c = v(&[None, Some(5), Some(4), Some(2), Some(3), Some(2)]);
        assert_eq!(fret_window(&c, 2, 24, 5), 1..=6);
    }

    #[test]
    fn window_clamped_to_short_neck() {
        let c = v(&[Some(11), Some(12), Some(12), None, None, None]);
        assert_eq!(fret_window(&c, 0, 12, 5), 10..=12);
    }

    #[test]
    fn window_min_span_extends_window() {
        let c = v(&[None, Some(3), Some(2), Some(0), Some(1), Some(0)]);
        assert_eq!(fret_window(&c, 0, 24, 5), 0..=5);
        assert_eq!(fret_window(&c, 0, 24, 6), 0..=6);
    }

    #[test]
    fn window_huge_span_does_not_overflow() {
        let c = v(&[None, Some(3), Some(2), Some(0), Some(1), Some(0)]);
        assert_eq!(fret_window(&c, 0, 24, u8::MAX), 0..=24);
    }

    #[test]
    fn window_all_muted_does_not_panic() {
        let c = v(&[None; 6]);
        assert_eq!(fret_window(&c, 0, 24, 5), 0..=5);
    }
}
