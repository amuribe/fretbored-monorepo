use crate::{instrument::FretBoardConfig, mapping::NoteCoordinate, theory::Note};
use std::fmt::Write;
use std::ops::RangeInclusive;

// Highest fret the renderer can draw: this is limited by a u64 bitmask
const MAX_FRET: u8 = 63;

/// Decides how each note on the fretboard is labeled. For example, a note can be labeled as a dot, with its name, or with its interval relative to the root of the chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelMode {
    #[default]
    NoteName,
    Dot,
    Interval,
}

/// Defines ANSII color codes for the fretboard elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScheme {
    /// Horizontal empty string segments (`───`)
    pub string_line: &'static str,
    /// Vertical fret dividers (`|`) and nut (`║`)
    pub fret_wire: &'static str,
    /// Fretted or open notes (`─●─`, `─C─`, etc.)
    pub active_note: &'static str,
    /// Root note emphasis (`─R─`, `─C─`, etc.)
    pub root_note: &'static str,
    /// Frets and wires behind a capo or outside the fret range are dimmed (`╌╌╌`)
    pub dimmed: &'static str,
    /// The capo bar (`─┃─`)
    pub capo: &'static str,
    /// Ansi reset code
    pub reset: &'static str,
}

impl ColorScheme {
    pub const DEFAULT_DARK: Self = Self {
        string_line: "\x1b[2;37m", // Dim white/gray so strings recede into background
        fret_wire: "\x1b[2;90m",   // Dark gray fret wires
        active_note: "\x1b[1;36m", // Bold Cyan
        root_note: "\x1b[1;33m",   // Bold Gold/Yellow
        dimmed: "\x1b[2;90m",      // Dimmed dark gray behind capo
        capo: "\x1b[1;35m",        // Bold Magenta
        reset: "\x1b[0m",
    };
    #[inline]
    pub fn colour(&self, buf: &mut String, enabled: bool, code: &str, text: &str) {
        if enabled && !code.is_empty() {
            buf.push_str(code);
            buf.push_str(text);
            buf.push_str(self.reset);
        } else {
            buf.push_str(text);
        }
    }
}
impl Default for ColorScheme {
    fn default() -> Self {
        Self::DEFAULT_DARK
    }
}

/// Display options for render_fretboard.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// The frets to draw, inclusive. Coordinates provided that are outside the range are ignored.
    pub fret_range: RangeInclusive<u8>,
    /// Root note is drawn with emphasis if provided. `None` renders all notes the same.
    pub root: Option<Note>,
    /// Spell accidentals as flats instead of sharps(default).
    pub use_flats: bool,
    /// Capo position. If provided, the fretboard is drawn as if a capo is placed at this fret and anything below it is dimmed, 0 means no capo.
    pub capo: u8,
    pub label_mode: LabelMode,
    pub show_inlays: bool,
    /// ANSI color codes are shown
    pub color: bool,
    /// Color scheme for the fretboard. Only used if `color` is true.
    pub scheme: ColorScheme,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            fret_range: 0..=12,
            root: None,
            use_flats: false,
            capo: 0,
            label_mode: LabelMode::default(),
            show_inlays: true,
            color: true,
            scheme: ColorScheme::default(),
        }
    }
}

/// Render the given coordinates onto an ASCII/Unicode fretboard diagram.
///
/// Positions not in `coords` are drawn as empty cells (`───`). A `fret_range` that extends beyond `config.fret_count` will be clamped.
/// Returns the diagram with no trailing newline
pub fn render_fretboard(
    config: &FretBoardConfig<'_>,
    coords: &[NoteCoordinate],
    options: &RenderOptions,
) -> String {
    // Allocate appropriate space depending on color option
    let capacity = if options.color { 2048 } else { 512 };
    let mut fretboard_str = String::with_capacity(capacity);

    let first_fret = *options.fret_range.start();
    // Clamp the last fret to the config's fret count
    let last_fret = (*options.fret_range.end())
        .min(config.fret_count)
        .min(MAX_FRET);
    // The effective fret range
    let frets = first_fret..=last_fret;

    // Prepopulate a bitmask of what's being fretted for quick lookup
    let mut active_frets = vec![0_u64; config.tuning.strings.len()];

    for coord in coords {
        let s_idx = coord.string_index as usize;
        // Only mark the fret as active if it's within the valid range of strings and frets
        if s_idx < active_frets.len() && frets.contains(&coord.fret) {
            active_frets[s_idx] |= 1_u64 << coord.fret;
        }
    }

    // Draw the fretboard header with fret numbers
    fretboard_str.push_str("   "); // 3 spaces matches the margin for the string labels
    for fret in frets.clone() {
        // 2-char right-aligned number + 1 space (cell right) + 1 space (fret wire) = 4 chars
        let _ = write!(fretboard_str, "{:>2}  ", fret);
    }
    fretboard_str.push('\n');

    // Iterate from the highest string to the lowest string (top to bottom)
    for (string_index, &active_mask) in active_frets.iter().enumerate().rev() {
        // Draw the string label (note name) at the start of the row
        let string_label = config
            .tuning
            .note_at_fret(string_index, 0)
            .map(|note| format_string_label(note, options.use_flats))
            .unwrap_or("?  ");
        fretboard_str.push_str(string_label);

        let scheme = &options.scheme;
        // Iterate over each fret on the string in the specified range and draw the fretboard cell
        for fret in frets.clone() {
            let is_active = (active_mask & (1_u64 << fret)) != 0;
            let below_capo = options.capo > 0 && fret < options.capo;
            let at_capo = options.capo > 0 && fret == options.capo;

            let note_opt = config.tuning.note_at_fret(string_index, fret);
            let is_root = matches!((options.root, note_opt), (Some(r), Some(n)) if r == n);

            // Draw the fret cell
            if is_active {
                let cell = match options.label_mode {
                    LabelMode::Dot => {
                        if is_root {
                            "─◆─"
                        } else {
                            "─●─"
                        }
                    }
                    LabelMode::NoteName => note_opt
                        .map(|note| format_note_cell(note, options.use_flats, is_root))
                        .unwrap_or("─X─"),
                    LabelMode::Interval => match (options.root, note_opt) {
                        (Some(root), Some(note)) => format_interval_cell(root, note),
                        (None, Some(note)) => format_note_cell(note, options.use_flats, false),
                        _ => "─X─",
                    },
                };

                // Determine how the cell should be colored based on the options and state
                let code = if below_capo {
                    scheme.dimmed
                } else if is_root {
                    scheme.root_note
                } else {
                    scheme.active_note
                };
                // Apply style to the current fret cell
                scheme.colour(&mut fretboard_str, options.color, code, cell);
            } else if at_capo {
                // Draw the capo bar
                scheme.colour(&mut fretboard_str, options.color, scheme.capo, "─┃─");
            } else if below_capo {
                // Draw dimmed string segments for frets below the capo
                scheme.colour(&mut fretboard_str, options.color, scheme.dimmed, "╌╌╌");
            } else {
                // Draw the empty string segment for frets above the capo
                scheme.colour(&mut fretboard_str, options.color, scheme.string_line, "───");
            }

            // Draw the fret wire/separator
            let sep = if fret == 0 || at_capo { "║" } else { "│" };
            let sep_code = if below_capo {
                scheme.dimmed
            } else {
                scheme.fret_wire
            };

            scheme.colour(&mut fretboard_str, options.color, sep_code, sep);
        }
        // Mark the end of the string row
        fretboard_str.push('\n');
    }

    // Draw the fret inlays if enabled
    if options.show_inlays {
        fretboard_str.push_str("   "); // 3 spaces matches the margin for the string labels
        for fret in frets.clone() {
            if fret > config.fret_count {
                break;
            }
            fretboard_str.push_str(format_inlay_cell(fret));
        }
        fretboard_str.push('\n');
    }

    // Remove the last newline for cleaner output
    fretboard_str.pop();

    fretboard_str
}

/// Helper function to return fret cell based on note
const fn format_note_cell(note: Note, use_flats: bool, is_root: bool) -> &'static str {
    if is_root {
        match (note, use_flats) {
            (Note::C, _) => "[C]",
            (Note::CSharp, false) => "[C#",
            (Note::CSharp, true) => "[Db",
            (Note::D, _) => "[D]",
            (Note::DSharp, false) => "[D#",
            (Note::DSharp, true) => "[Eb",
            (Note::E, _) => "[E]",
            (Note::F, _) => "[F]",
            (Note::FSharp, false) => "[F#",
            (Note::FSharp, true) => "[Gb",
            (Note::G, _) => "[G]",
            (Note::GSharp, false) => "[G#",
            (Note::GSharp, true) => "[Ab",
            (Note::A, _) => "[A]",
            (Note::ASharp, false) => "[A#",
            (Note::ASharp, true) => "[Bb",
            (Note::B, _) => "[B]",
        }
    } else {
        match (note, use_flats) {
            (Note::C, _) => "─C─",
            (Note::CSharp, false) => "─C#",
            (Note::CSharp, true) => "─Db",
            (Note::D, _) => "─D─",
            (Note::DSharp, false) => "─D#",
            (Note::DSharp, true) => "─Eb",
            (Note::E, _) => "─E─",
            (Note::F, _) => "─F─",
            (Note::FSharp, false) => "─F#",
            (Note::FSharp, true) => "─Gb",
            (Note::G, _) => "─G─",
            (Note::GSharp, false) => "─G#",
            (Note::GSharp, true) => "─Ab",
            (Note::A, _) => "─A─",
            (Note::ASharp, false) => "─A#",
            (Note::ASharp, true) => "─Bb",
            (Note::B, _) => "─B─",
        }
    }
}

/// Helper function to return fret cell based on interval
const fn format_interval_cell(root: Note, target: Note) -> &'static str {
    let semitones = (target.value() + 12 - root.value()) % 12;
    match semitones {
        0 => "─R─",  // Unison / Root
        1 => "─b2",  // MinorSecond
        2 => "─2─",  // MajorSecond
        3 => "─b3",  // MinorThird
        4 => "─3─",  // MajorThird
        5 => "─4─",  // PerfectFourth
        6 => "─b5",  // Tritone
        7 => "─5─",  // PerfectFifth
        8 => "─b6",  // MinorSixth
        9 => "─6─",  // MajorSixth
        10 => "─b7", // MinorSeventh
        11 => "─7─", // MajorSeventh
        _ => unreachable!(),
    }
}

/// Helper function to return the string label (note followed by a space)
const fn format_string_label(note: Note, use_flats: bool) -> &'static str {
    match (note, use_flats) {
        (Note::C, _) => "C  ",
        (Note::CSharp, false) => "C# ",
        (Note::CSharp, true) => "Db ",
        (Note::D, _) => "D  ",
        (Note::DSharp, false) => "D# ",
        (Note::DSharp, true) => "Eb ",
        (Note::E, _) => "E  ",
        (Note::F, _) => "F  ",
        (Note::FSharp, false) => "F# ",
        (Note::FSharp, true) => "Gb ",
        (Note::G, _) => "G  ",
        (Note::GSharp, false) => "G# ",
        (Note::GSharp, true) => "Ab ",
        (Note::A, _) => "A  ",
        (Note::ASharp, false) => "A# ",
        (Note::ASharp, true) => "Bb ",
        (Note::B, _) => "B  ",
    }
}

/// Helper function for formatting fret inlay
///
/// Frets 3, 5, 7, 9, 15, 17, 19 and 21 have a single inlay dot, while fret 12 and 24 have a double inlay dot. All other frets have no inlay.
const fn format_inlay_cell(fret: u8) -> &'static str {
    match fret {
        3 | 5 | 7 | 9 | 15 | 17 | 19 | 21 => " •  ",
        12 | 24 => " •• ",
        _ => "    ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instrument::{GuitarTunings, Orientation, StandardTunings},
        theory::Note,
    };

    #[test]
    fn test_render_fretboard() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            12,
            Orientation::RightHanded,
        );
        let coords = vec![
            NoteCoordinate {
                string_index: 0,
                fret: 3,
            }, // G
            NoteCoordinate {
                string_index: 1,
                fret: 2,
            }, // B
            NoteCoordinate {
                string_index: 2,
                fret: 0,
            }, // E
            NoteCoordinate {
                string_index: 3,
                fret: 0,
            }, // A
            NoteCoordinate {
                string_index: 4,
                fret: 3,
            }, // D
            NoteCoordinate {
                string_index: 5,
                fret: 3,
            }, // G
        ];
        let options = RenderOptions {
            root: Some(Note::G),
            label_mode: LabelMode::Dot,
            ..Default::default()
        };
        let diagram = render_fretboard(&config, &coords, &options);
        println!("{}\n", diagram);
    }

    #[test]
    fn test_render_interval_label_mode() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            12,
            Orientation::RightHanded,
        );
        let coords = vec![
            NoteCoordinate {
                string_index: 0,
                fret: 3,
            }, // G
            NoteCoordinate {
                string_index: 1,
                fret: 2,
            }, // B
            NoteCoordinate {
                string_index: 2,
                fret: 0,
            }, // E
            NoteCoordinate {
                string_index: 3,
                fret: 0,
            }, // A
            NoteCoordinate {
                string_index: 4,
                fret: 3,
            }, // D
            NoteCoordinate {
                string_index: 5,
                fret: 3,
            }, // G
        ];
        // Add a root note to the options so that intervals can be calculated
        let options = RenderOptions {
            label_mode: LabelMode::Interval,
            root: Some(Note::G),
            ..Default::default()
        };
        let diagram = render_fretboard(&config, &coords, &options);
        println!("{}\n", diagram);
    }

    #[test]
    fn test_render_note_name_label_mode() {
        let config = FretBoardConfig::new(
            GuitarTunings::guitar_eb_standard(),
            12,
            Orientation::RightHanded,
        );
        let coords = vec![
            NoteCoordinate {
                string_index: 0,
                fret: 3,
            }, // G
            NoteCoordinate {
                string_index: 1,
                fret: 2,
            }, // B
            NoteCoordinate {
                string_index: 2,
                fret: 0,
            }, // E
            NoteCoordinate {
                string_index: 3,
                fret: 0,
            }, // A
            NoteCoordinate {
                string_index: 4,
                fret: 3,
            }, // D
            NoteCoordinate {
                string_index: 5,
                fret: 3,
            }, // G
        ];

        let options = RenderOptions {
            label_mode: LabelMode::NoteName,
            ..Default::default()
        };
        let diagram = render_fretboard(&config, &coords, &options);
        println!("{}\n", diagram);
    }

    #[test]
    fn test_render_with_capo_and_color() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            12,
            Orientation::RightHanded,
        );
        // A Major using an open G-shape with a capo at fret 2
        let coords = vec![
            NoteCoordinate {
                string_index: 0,
                fret: 5,
            }, // A (Root)
            NoteCoordinate {
                string_index: 1,
                fret: 4,
            }, // C# (3rd)
            NoteCoordinate {
                string_index: 2,
                fret: 2,
            }, // E (5th, resting on the capo)
            NoteCoordinate {
                string_index: 3,
                fret: 2,
            }, // A (Root, resting on the capo)
            NoteCoordinate {
                string_index: 4,
                fret: 5,
            }, // E (5th)
            NoteCoordinate {
                string_index: 5,
                fret: 5,
            }, // A (Root)
        ];

        let options = RenderOptions {
            label_mode: LabelMode::Interval,
            root: Some(Note::A),
            capo: 2,
            color: true,
            ..Default::default()
        };

        let diagram = render_fretboard(&config, &coords, &options);
        println!("{}\n", diagram);

        // Verify capo and dimmed region are present in the diagram
        assert!(
            diagram.contains("╌╌╌"),
            "Frets below capo should use dotted lines"
        );
        assert!(
            diagram.contains("─┃─"),
            "Unfretted capo positions should show the capo bar"
        );
    }

    #[test]
    fn test_render_invariants_and_clamping() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            12,
            Orientation::RightHanded,
        );
        let coords = vec![
            NoteCoordinate {
                string_index: 0,
                fret: 1,
            }, // Behind capo (filtered out)
            NoteCoordinate {
                string_index: 0,
                fret: 5,
            }, // Root A
        ];
        let options = RenderOptions {
            fret_range: 0..=20, // Exceeds config.fret_count (12) -> must clamp to 12
            root: Some(Note::A),
            capo: 2,
            label_mode: LabelMode::Interval,
            color: true,
            ..Default::default()
        };

        let diagram = render_fretboard(&config, &coords, &options);

        let dim = ColorScheme::DEFAULT_DARK;
        assert!(diagram.contains(&format!("{}─b6{}", dim.dimmed, dim.reset)));
        assert!(diagram.contains(&format!("{}─R─{}", dim.root_note, dim.reset)));
    }
    #[test]
    fn test_fret_range_beyond_bitmask_does_not_panic() {
        let config = FretBoardConfig::new(
            StandardTunings::guitar_standard(),
            100,
            Orientation::RightHanded,
        );
        let coords = [NoteCoordinate {
            string_index: 0,
            fret: 40,
        }];
        let options = RenderOptions {
            fret_range: 0..=100,
            color: false,
            ..Default::default()
        };
        let diagram = render_fretboard(&config, &coords, &options);
        assert!(diagram.lines().count() >= 7); // header + 6 strings
    }
}
