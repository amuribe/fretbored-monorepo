pub mod registry;
pub mod tuning;
pub use tuning::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// If Orientation LeftHanded, reverse the Tuning slice
pub enum Orientation {
    #[default]
    RightHanded,
    LeftHanded,
}

// Because it holds Tuning<'a>, must be <'a>
pub struct FretBoardConfig<'a> {
    pub tuning: Tuning<'a>,
    pub fret_count: u8,
    pub orientation: Orientation,
}

impl<'a> FretBoardConfig<'a> {
    pub fn new(tuning: Tuning<'a>, fret_count: u8, orientation: Orientation) -> Self {
        Self {
            tuning,
            fret_count,
            orientation,
        }
    }
}

#[test]
fn standard_guitar_test() {
    let guitar = FretBoardConfig::new(
        StandardTunings::guitar_standard(),
        24,
        Orientation::RightHanded,
    );
    assert_eq!(guitar.tuning, StandardTunings::guitar_standard());
    assert_eq!(guitar.fret_count, 24);
    assert_eq!(guitar.orientation, Orientation::RightHanded);
}
