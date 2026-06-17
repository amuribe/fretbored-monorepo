use crate::mapping::NoteCoordinate;
use crate::theory::Note;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashSet;

// A single playable chord shape, each index = one string on the instrument
// Some(fret) = play that fret, (0 = open string) None = mute this string
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Voicing {
    pub frets: Vec<Option<u8>>,
}

// Runtime config derived from the active instrument
#[derive(Debug, Clone)]
pub struct VoicingConfig {
    // Max distance between the lowest fretted note and highest fretted note (excluding open strings).
    // e.g guitar = 5
    pub max_stretch: u8,

    // When true, every note in the target chord must appear in the voicing
    // When false, partial voicings (e.g. omiting the fifth) are allowed
    pub require_all_target_notes: bool,

    pub allow_open_strings: bool,
    // The fret number where the capo is placed. 0 means no capo (the nut).
    // All fretted notes must be >= capo_fret; you cannot fret behind a capo.
    pub capo_fret: u8,
}

impl Default for VoicingConfig {
    fn default() -> Self {
        Self {
            max_stretch: 4,
            require_all_target_notes: false,
            allow_open_strings: true,
            capo_fret: 0,
        }
    }
}

/// Entry point for voicing generation.
///
/// Strategy overview:
/// 1. Pre-group all valid frets by string for O(1) lookup during backtracking.
/// 2. Slide a window of width `max_stretch` across the fretboard, from
///    (capo + 1) up to the highest available fret.
/// 3. For each window position, run a backtracking search that assigns each
///    string to one of: muted, open (if a capo open = capo fret), or a fret within the window.
/// 4. Each complete combination is validated and, if valid, inserted into the
///    result set. HashSet deduplication handles any overlap between windows.
pub fn generate_voicing(
    target_notes: &[Note],
    raw_coords: &[NoteCoordinate],
    string_count: usize,
    open_strings: &[u8],
    voice_config: &VoicingConfig,
) -> HashSet<Voicing> {
    // Group coordinates by string for O(1) lookups,
    // to avoid scanning the full coordinate matrix on every recursive call
    let mut valid_frets_per_string: Vec<Vec<u8>> = vec![vec![]; string_count];
    let mut highest_available_fret = 0;

    for coord in raw_coords {
        valid_frets_per_string[coord.string_index as usize].push(coord.fret);
        if coord.fret > highest_available_fret {
            highest_available_fret = coord.fret;
        }
    }

    // You can't fret behind a capo, so first valid fretted position
    // is capo_fret + 1.  Fret 0 (open string) is handled separately.
    let search_start = max(1, voice_config.capo_fret + 1);

    // saturating sub returns 0 if result is negative
    let search_end = highest_available_fret
        .saturating_sub(voice_config.max_stretch)
        .max(search_start); // ensures at least one iteration

    // Convert target notes into a 12-bit integer (e.g., C Major = 0b10010001)
    let mut target_mask: u16 = 0;
    for note in target_notes {
        target_mask |= 1 << note.value();
    }
    let mut results: HashSet<Voicing> = HashSet::new();

    // Slide window from search_start to search_end (inclusive)

    for window_start in search_start..=search_end {
        let window_end = window_start + voice_config.max_stretch;
        // current_combo holds the in-progress assignment for each string.
        // None = muted. It is mutated in place and cloned only when a valid
        // voicing is found, keeping allocation costs low.
        let mut current_combo: Vec<Option<u8>> = vec![None; string_count];
        backtrack(
            0, // start from the lowest string (index 0)
            string_count,
            &valid_frets_per_string,
            window_start,
            window_end,
            &mut current_combo,
            &mut results,
            target_mask,
            open_strings,
            voice_config,
        );
    }

    results
}

/// Recursive backtracking: assigns a value to each string one at a time.
///
/// At each depth level current_string we explore two branches:
///   Branch 1 — mute: set current_combo[current_string] = None, recurse.
///   Branch 2 — play: for each fret on this string that passes pruning,
///              set current_combo[current_string] = Some(fret), recurse.
///
/// After returning from a branch implicitly undo the assignment by
/// overwriting current_combo[current_string] on the next iteration
fn backtrack(
    current_string: usize,
    total_strings: usize,
    valid_frets_per_string: &[Vec<u8>],
    window_start: u8,
    window_end: u8,
    current_combo: &mut Vec<Option<u8>>,
    results: &mut HashSet<Voicing>,
    target_mask: u16,
    open_strings: &[u8],
    config: &VoicingConfig,
) {
    // Base case (bottom of tree) combination is complete
    if current_string == total_strings {
        if is_valid_chord(current_combo, target_mask, open_strings, config) {
            results.insert(Voicing {
                frets: current_combo.clone(),
            });
        }
        return;
    }

    // Branch 1 Mute the current string
    // Assign None means don't play this string
    current_combo[current_string] = None;
    backtrack(
        current_string + 1,
        total_strings,
        valid_frets_per_string,
        window_start,
        window_end,
        current_combo,
        results,
        target_mask,
        open_strings,
        config,
    );

    // Branch 2: Play a fret on the current string
    // Each valid fret for the string is tried, subject to two pruning checks.
    for &fret in &valid_frets_per_string[current_string] {
        // Pruning rule 1: never fret behind a capo
        if fret < config.capo_fret {
            continue;
        }

        // If fret is considered an open string to a capo
        let is_open_to_capo = fret == config.capo_fret;

        // Pruning rule 2: fretted notes must lie within the current window
        // so that the hand-stretch constraint is respected.
        let is_in_window = fret >= window_start && fret <= window_end;

        // Only proceed if this fret is reachable (either open-to-capo when
        // open strings are allowed, or within the sliding window).
        if (is_open_to_capo && config.allow_open_strings) || is_in_window {
            current_combo[current_string] = Some(fret);
            backtrack(
                current_string + 1,
                total_strings,
                valid_frets_per_string,
                window_start,
                window_end,
                current_combo,
                results,
                target_mask,
                open_strings,
                config,
            );
        }
    }
}

/// Validates that a fully-constructed chord shape is musically usable.
///
/// Three rules must all pass:
///   1. At least 3 strings are played (a chord, not a single note or dyad).
///   2. No played note falls outside the target pitch-class set ("no wrong notes").
///   3. Optionally, every target note appears at least once (if specified in config)
fn is_valid_chord(
    chord_shape: &[Option<u8>],
    target_mask: u16,    // bits set for each desired pitch class
    open_strings: &[u8], // used as tuning e.g., [4, 9, 2, 7, 11, 4] (E A D G B E as integers 0-11)
    config: &VoicingConfig,
) -> bool {
    let mut played_count = 0;
    let mut played_mask: u16 = 0;

    for (string_idx, &fret_opt) in chord_shape.iter().enumerate() {
        if let Some(fret) = fret_opt {
            played_count += 1;

            let pitch_class = (open_strings[string_idx] as u16 + fret as u16) % 12;

            // Record this pitch class as played
            played_mask |= 1 << pitch_class;
        }
    }

    // Rule 1: need at least 3 notes to form a chord.
    if played_count < 3 {
        return false;
    }

    // Rule 2: no played note may be outside the target set.
    // played_mask & !target_mask isolates any "wrong note" bits.
    // If that expression is non-zero, at least one wrong note is present.
    if (played_mask & !target_mask) != 0 {
        return false;
    }

    // Rule 3: every target pitch class must be present.
    // played_mask & target_mask keeps only the covered target bits.
    // If this doesn't equal target_mask, at least one target note is missing.
    if config.require_all_target_notes && (played_mask & target_mask) != target_mask {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to generate a matrix of available coordinates for standard tuning
    fn generate_test_board(strings: usize, frets: u8) -> Vec<NoteCoordinate> {
        let mut coords = Vec::new();
        for string_index in 0..strings {
            for fret in 0..=frets {
                coords.push(NoteCoordinate {
                    string_index: string_index as u8,
                    fret,
                });
            }
        }
        coords
    }

    #[test]
    fn test_minimum_three_notes_required() {
        let coords = generate_test_board(6, 12);
        let open_strings = [4, 9, 2, 7, 11, 4]; // E A D G B E
        let config = VoicingConfig {
            max_stretch: 4,
            require_all_target_notes: false,
            allow_open_strings: false,
            capo_fret: 0,
        };

        let results = generate_voicing(&[], &coords, 6, &open_strings, &config);

        for voicing in results {
            let active_notes = voicing.frets.iter().filter(|f| f.is_some()).count();
            assert!(active_notes >= 3, "Found a voicing with less than 3 notes");
        }
    }

    #[test]
    fn test_max_stretch_respected() {
        let coords = generate_test_board(6, 15);
        let open_strings = [4, 9, 2, 7, 11, 4];
        let config = VoicingConfig {
            max_stretch: 3, // Tight stretch
            require_all_target_notes: false,
            allow_open_strings: false, // Isolate fretted notes
            capo_fret: 0,
        };

        let results = generate_voicing(&[], &coords, 6, &open_strings, &config);

        for voicing in results {
            let frets: Vec<u8> = voicing.frets.iter().filter_map(|&f| f).collect();
            if frets.is_empty() {
                continue;
            }

            let min_fret = frets.iter().min().unwrap();
            let max_fret = frets.iter().max().unwrap();

            assert!(
                (max_fret - min_fret) <= config.max_stretch,
                "Stretch exceeded: {:?}",
                frets
            );
        }
    }

    #[test]
    fn test_target_mask_completeness() {
        let open_strings = [4, 9, 2, 7, 11, 4];

        let config = VoicingConfig {
            max_stretch: 4,
            require_all_target_notes: true, // Strict validation
            allow_open_strings: true,
            capo_fret: 0,
        };

        // Target: C (0), E (4), G (7)
        let target_mask: u16 = (1 << 0) | (1 << 4) | (1 << 7);

        // Valid shape: E C E G C E -> [0, 3, 2, 0, 1, 0]
        let valid_shape = vec![Some(0), Some(3), Some(2), Some(0), Some(1), Some(0)];
        assert!(
            is_valid_chord(&valid_shape, target_mask, &open_strings, &config),
            "Valid C major chord was rejected"
        );

        // Invalid shape: E C E G B E -> [0, 3, 2, 0, 0, 0] (B is not in C Major)
        let invalid_shape = vec![Some(0), Some(3), Some(2), Some(0), Some(0), Some(0)];
        assert!(
            !is_valid_chord(&invalid_shape, target_mask, &open_strings, &config),
            "Invalid chord containing a B was accepted"
        );
    }

    #[test]
    fn test_capo_logic() {
        let coords = generate_test_board(6, 12);
        let open_strings = [4, 9, 2, 7, 11, 4];
        let capo = 5;

        let config = VoicingConfig {
            max_stretch: 4,
            require_all_target_notes: false,
            allow_open_strings: true,
            capo_fret: capo,
        };

        let results = generate_voicing(&[], &coords, 6, &open_strings, &config);

        for voicing in results {
            for &fret_opt in &voicing.frets {
                if let Some(fret) = fret_opt {
                    assert!(
                        fret >= capo,
                        "Found a fret ({}) below the capo ({})",
                        fret,
                        capo
                    );
                }
            }
        }
    }
}
