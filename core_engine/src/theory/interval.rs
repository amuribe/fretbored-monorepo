use crate::theory::Note;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    Unison,
    MinorSecond,
    MajorSecond,
    MinorThird,
    MajorThird,
    PerfectFourth,
    Tritone,
    PerfectFifth,
    MinorSixth,
    MajorSixth,
    MinorSeventh,
    MajorSeventh,
    PerfectOctave,
}

impl Interval {
    // Maps interval to coresponding semitone distance(u8)
    pub fn semitones(&self) -> u8 {
        match self {
            Interval::Unison => 0,
            Interval::MinorSecond => 1,
            Interval::MajorSecond => 2,
            Interval::MinorThird => 3,
            Interval::MajorThird => 4,
            Interval::PerfectFourth => 5,
            Interval::Tritone => 6,
            Interval::PerfectFifth => 7,
            Interval::MinorSixth => 8,
            Interval::MajorSixth => 9,
            Interval::MinorSeventh => 10,
            Interval::MajorSeventh => 11,
            Interval::PerfectOctave => 12,
        }
    }
}

// Pass by value: Note and Interval implement Copy and are smaller than a pointer.
pub fn calculate_target_note(root: Note, interval: Interval) -> Note {
    // The value of the root (e.g C = 0, G = 7)
    let root_val = root.value();
    // The semitone distance of the interval (e.g Unison = 0, PerfectFifth = 7)
    let interval_val = interval.semitones();

    // The value of the target note (e.g G + Major Third = 7 + 4 = 11 -> B)
    let target_val = (root_val + interval_val) % 12;

    Note::from_midi(target_val)
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_addition() {
        // G + Perfect 5th = D
        assert_eq!(
            calculate_target_note(Note::G, Interval::PerfectFifth),
            Note::D
        );

        // C + Major 3rd = E
        assert_eq!(
            calculate_target_note(Note::C, Interval::MajorThird),
            Note::E
        );

        // B + Minor 2nd = C (Tests modulo wrap: 11 + 1 = 12 % 12 = 0)
        assert_eq!(
            calculate_target_note(Note::B, Interval::MinorSecond),
            Note::C
        );
    }
}
