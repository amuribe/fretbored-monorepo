use crate::theory::Note;
use crate::theory::interval::{Interval, calculate_target_note};

// Use composition and separate the chord's quality(triad/sus) from the extension
// It is slightly slower but the goal of this project is to handle extremely complex chords
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
}

// The chord's extension m7 m9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extension {
    // Not all chords have extensions i.e triads
    None,
    // 6ths
    Sixth,
    SixNine,
    // 7ths
    MinorSeventh,
    MajorSeventh,
    DominantSeventh,
    DiminishedSeventh,
    // 9ths
    Ninth,
    MajorNinth,
    FlatNinth,
    SharpNinth,
    // 11ths
    Eleventh,
    MajorEleventh,
    SharpEleventh,
    MajorSharpEleventh,
    // 13ths
    Thirteenth,
    MajorThirteenth,
    FlatThirteenth,
}

// ChordType is the abstract mathematical formula of the chord
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChordType {
    pub quality: ChordQuality,
    pub extension: Extension,
}

// Chord is the concrete instance of the chord
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub root: Note,
    pub chord_type: ChordType,
    pub notes: Vec<Note>,
}

impl ChordQuality {
    // Generate a static slice of the intervals after root which make up ChordQuality (e.g. minor: m3, P5 )
    pub fn intervals(&self) -> &'static [Interval] {
        match self {
            ChordQuality::Major => &[Interval::MajorThird, Interval::PerfectFifth],
            ChordQuality::Minor => &[Interval::MinorThird, Interval::PerfectFifth],
            ChordQuality::Diminished => &[Interval::MinorThird, Interval::Tritone],
            ChordQuality::Augmented => &[Interval::MajorThird, Interval::MinorSixth],
            ChordQuality::Sus2 => &[Interval::MajorSecond, Interval::PerfectFifth],
            ChordQuality::Sus4 => &[Interval::PerfectFourth, Interval::PerfectFifth],
        }
    }
}

impl Extension {
    // Generate a static slice of the intervals which make up the extension (e.g MinorSeventh: {MinorSeventh})
    pub fn intervals(&self) -> &'static [Interval] {
        match self {
            Extension::None => &[],

            // 6ths (Adds the Major 6th interval)
            Extension::Sixth => &[Interval::MajorSixth],
            // 6/9 (Stacks Major 6th and the 9th,  9th maps to a Major 2nd)
            Extension::SixNine => &[Interval::MajorSixth, Interval::MajorSecond],

            // 7ths
            Extension::MinorSeventh | Extension::DominantSeventh => &[Interval::MinorSeventh],
            Extension::MajorSeventh => &[Interval::MajorSeventh],
            Extension::DiminishedSeventh => &[Interval::MajorSixth], // Enharmonically bb7

            // 9ths (Includes the 7th. Modulo 12 maps 9th to 2nd)
            Extension::Ninth => &[Interval::MinorSeventh, Interval::MajorSecond],
            Extension::MajorNinth => &[Interval::MajorSeventh, Interval::MajorSecond],
            Extension::FlatNinth => &[Interval::MinorSeventh, Interval::MinorSecond],
            Extension::SharpNinth => &[Interval::MinorSeventh, Interval::MinorThird],

            // 11ths (Includes 7th and 9th. 11th maps to 4th)
            Extension::Eleventh => &[
                Interval::MinorSeventh,
                Interval::MajorSecond,
                Interval::PerfectFourth,
            ],
            Extension::MajorEleventh => &[
                Interval::MajorSeventh,
                Interval::MajorSecond,
                Interval::PerfectFourth,
            ],
            Extension::SharpEleventh => &[
                Interval::MinorSeventh,
                Interval::MajorSecond,
                Interval::Tritone,
            ],
            Extension::MajorSharpEleventh => &[
                Interval::MajorSeventh,
                Interval::MajorSecond,
                Interval::Tritone,
            ],
            // 13ths (Includes 7th and 9th. 11th is omitted. 13th maps to 6th)
            Extension::Thirteenth => &[
                Interval::MinorSeventh,
                Interval::MajorSecond,
                Interval::MajorSixth,
            ],
            Extension::MajorThirteenth => &[
                Interval::MajorSeventh,
                Interval::MajorSecond,
                Interval::MajorSixth,
            ],
            Extension::FlatThirteenth => &[
                Interval::MinorSeventh,
                Interval::MajorSecond,
                Interval::MinorSixth,
            ],
        }
    }
}

impl Chord {
    // Allocate memory and calculate notes based on ChordQuality and Extension
    pub fn new(root: Note, chord_type: ChordType) -> Self {
        // The intervals that make up the chord quality (e.g minor -> minor third, perfect fifth)
        let quality_intervals = chord_type.quality.intervals();
        // The intervals that make up the chord extension (e.g major9 -> major7, major9 (mapped to major 2nd))
        let extension_intervals = chord_type.extension.intervals();

        // Get amount of notes in the chord (len(quality) + len(extension) + len(root = 1))
        let mut notes = Vec::with_capacity(quality_intervals.len() + extension_intervals.len() + 1);
        // Add root to start of chord
        notes.push(root);

        // Stack triad/sus intervals
        // deref interval
        for &interval in quality_intervals {
            notes.push(calculate_target_note(root, interval));
        }

        // Stack extension intervals
        // deref interval
        for &interval in extension_intervals {
            notes.push(calculate_target_note(root, interval));
        }

        Self {
            root,
            chord_type,
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_major_triad() {
        let c_major = Chord::new(
            Note::C,
            ChordType {
                quality: ChordQuality::Major,
                extension: Extension::None,
            },
        );
        assert_eq!(c_major.notes, vec![Note::C, Note::E, Note::G]);
        assert_eq!(c_major.root, Note::C);
    }

    #[test]
    fn test_generate_dominant_ninth() {
        // G9 = G, B, D, F, A
        let g_dom9 = Chord::new(
            Note::G,
            ChordType {
                quality: ChordQuality::Major,
                extension: Extension::Ninth,
            },
        );
        assert_eq!(
            g_dom9.notes,
            vec![Note::G, Note::B, Note::D, Note::F, Note::A]
        );
    }

    #[test]
    fn test_generate_minor_thirteenth() {
        // A min13 = A, C, E, G, B, F#
        let a_min13 = Chord::new(
            Note::A,
            ChordType {
                quality: ChordQuality::Minor,
                extension: Extension::Thirteenth,
            },
        );
        assert_eq!(
            a_min13.notes,
            vec![Note::A, Note::C, Note::E, Note::G, Note::B, Note::FSharp]
        );
    }
}
