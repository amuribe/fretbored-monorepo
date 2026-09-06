use serde::{Deserialize, Serialize};

use crate::voicing::Voicing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankingStrategy {
    MostPlayable,
    LowestPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoicingScore {
    voicing: Voicing,
    score: i32,
}

fn score_voicing(voicing: &Voicing) -> i32 {
    // Filter out unfretted strings
    let fretted: Vec<u8> = voicing.frets.iter().flatten().copied().collect();
    // All open voicing are considered easiest
    if fretted.is_empty() {
        return 0;
    }
    // Calculate the largest stretch in the voicing
    let max_fret = *fretted.iter().max().unwrap() as i32;
    let min_fret = *fretted.iter().min().unwrap() as i32;
    let stretch = max_fret - min_fret;
    // Count the number of Open strings (None(s) in the vec)
    let open_count = voicing.frets.iter().filter(|x| x.is_none()).count() as i32;

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
        .iter()
        .map(|v| VoicingScore {
            voicing: v.clone(),
            score: score_voicing(v),
        })
        .collect();

    scored.sort_by_key(|v| std::cmp::Reverse(v.score));

    match strategy {
        RankingStrategy::MostPlayable => scored,
        RankingStrategy::LowestPosition => {
            // Sort voicings solely by the lowest fret instead
            scored.sort_by_key(|vs| {
                vs.voicing
                    .frets
                    .iter()
                    .flatten()
                    .min()
                    .copied()
                    .unwrap_or(0)
            });
            scored
        }
    }
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
