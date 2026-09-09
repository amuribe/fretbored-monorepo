use serde::{Deserialize, Serialize};

use crate::voicing::{Voicing, calculate_open_count};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankingStrategy {
    MostPlayable,
    LowestPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoicingScore {
    pub voicing: Voicing,
    pub score: i32,
}

/// Compute (min_fret, max_fret) for fretted notes (> 0) in a single pass.
/// Returns None if there are no fretted notes
pub fn min_max_fret(voicing: &Voicing) -> Option<(u8, u8)> {
    voicing
        .frets
        .iter()
        .filter_map(|&f| f)
        .filter(|&f| f > 0)
        .fold(None, |acc, f| match acc {
            None => Some((f, f)),
            Some((min, max)) => Some((min.min(f), max.max(f))),
        })
}

/// Calculate the physical finger stretch of playing a chord (excluding open and muted strings)
#[inline]
pub fn calculate_stretch(voicing: &Voicing) -> u8 {
    // Calculate stretch using min and max fret in voicing, stretch = 0 if no notes are fretted
    match min_max_fret(voicing) {
        Some((min, max)) => max - min,
        None => 0,
    }
}

#[inline]
pub fn min_fret_in_voicing(voicing: &Voicing) -> u8 {
    min_max_fret(voicing).map(|(min, _)| min).unwrap_or(0)
}

#[inline]
pub fn max_fret_in_voicing(voicing: &Voicing) -> u8 {
    min_max_fret(voicing).map(|(_, max)| max).unwrap_or(0)
}

fn score_voicing(voicing: &Voicing) -> i32 {
    let (min_fret, max_fret) = match min_max_fret(voicing) {
        Some((min, max)) => (min as i32, max as i32),
        None => return 0, // All open or muted: stretch is 0
    };

    let stretch = max_fret - min_fret;
    let open_count = calculate_open_count(voicing) as i32;

    // Scoring based off: open string count, how difficult the stretch is, and how low it is on the neck
    // TBD: Scoring Values might change
    // stretch_penalty: -stretch * 2
    // open_bonus: open_count * 3
    // position_bonus: -(lowest_fret / 2)
    // scoring formula: stretch_penalty + open_bonus + position_bonus
    let stretch_penalty = -stretch * 2;
    let open_bonus = open_count * 3;
    let position_bonus = -min_fret / 2;

    // Score the voicing
    stretch_penalty + open_bonus + position_bonus
}

pub fn rank_voicings(
    voicings: std::collections::HashSet<Voicing>,
    strategy: RankingStrategy,
) -> Vec<VoicingScore> {
    let mut scored: Vec<VoicingScore> = voicings
        .into_iter()
        .map(|v| {
            let score = score_voicing(&v);
            VoicingScore { voicing: v, score }
        })
        .collect();

    match strategy {
        RankingStrategy::MostPlayable => scored.sort_by_key(|v| std::cmp::Reverse(v.score)),
        RankingStrategy::LowestPosition => {
            // Sort voicings solely by the lowest fret instead
            scored.sort_by_key(|vs| min_fret_in_voicing(&vs.voicing));
        }
    }

    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voicing::Voicing;

    #[test]
    fn test_score_voicing_open_strings_bonus() {
        let voicing_open = Voicing {
            frets: vec![Some(0), Some(2), None, None, None, None],
        };
        let voicing_closed = Voicing {
            frets: vec![Some(5), Some(7), Some(7), Some(5), Some(5), Some(5)],
        };
        // Open should score higher
        assert!(score_voicing(&voicing_open) > score_voicing(&voicing_closed));
    }

    #[test]
    fn test_all_open_voicing() {
        let voicing = Voicing {
            frets: vec![None, None, None, None, None, None],
        };
        assert_eq!(score_voicing(&voicing), 0);
    }
}
