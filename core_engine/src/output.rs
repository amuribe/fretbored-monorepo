// Format a voicing as a readable String output

use crate::{
    instrument::Tuning,
    voicing::{Voicing, calculate_open_count},
};

// Holds a voicing and metadata formatted for display.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoicingDisplay<'a> {
    pub voicing: Voicing,
    pub tuning: &'a Tuning<'a>,
    pub note_names: Vec<String>,
}

impl<'a> VoicingDisplay<'a> {
    pub fn new(voicing: Voicing, tuning: &'a Tuning<'a>) -> Self {
        let note_names = voicing
            .frets
            .iter()
            .enumerate()
            .map(|(string_idx, fret_opt)| match fret_opt {
                Some(fret) => tuning
                    .note_at_fret(string_idx, *fret)
                    .map(|n| format!("{:?}", n))
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

pub fn format_voicing(voicing: &Voicing, tuning: &Tuning) -> String {
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
    let note_names: Vec<String> = voicing
        .frets
        .iter()
        .enumerate()
        .map(|(string_idx, fret_opt)| match fret_opt {
            Some(fret) => tuning
                .note_at_fret(string_idx, *fret)
                .map(|n| format!("{:?}", n))
                .unwrap_or_else(|| "X".to_string()),
            None => "-".to_string(),
        })
        .collect();

    format!("Frets: {} | Notes: {}", fret_str, note_names.join(" "))
}

pub fn format_voicing_w_playability(voicing: &Voicing, tuning: &Tuning) -> String {
    // Format voicing to string with plauability information
    let base_str = format_voicing(voicing, tuning);
    let fretted: Vec<u8> = voicing.frets.iter().flatten().copied().collect();

    let stretch = if fretted.is_empty() {
        0
    } else {
        // Max fret - min fret
        fretted.iter().max().unwrap() - fretted.iter().min().unwrap()
    };

    let open_count = calculate_open_count(voicing);
    format!("{} | Stretch: {} | Open: {}", base_str, stretch, open_count)
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
        let formatted = format_voicing(&voicing, &tuning);
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
        let formatted = format_voicing_w_playability(&voicing, &tuning);
        assert!(formatted.contains("Stretch: 3"));
        assert!(
            formatted.contains("Open: 2"),
            "Expected 'Open: 2', Actual: {}",
            formatted
        );
    }
}
