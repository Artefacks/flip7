//! Totaux, annulation et gagnant d'une partie de Flip 7.
//! Chaque manche ne contient que le score annoncé. Un joueur éliminé marque 0.

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 18;
pub const MAX_NAME_CHARS: usize = 24;
pub const MAX_ROUND_SCORE: i32 = 999;
pub const MAX_TARGET: i32 = 9999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameError {
    NeedTwoPlayers,
    TooManyPlayers,
    EmptyName,
    DuplicateName,
    NameTooLong,
    BadTarget,
    MissingScore,
    BadScore,
    NegativeScore,
    ScoreTooHigh,
    ScoreCount,
    NothingToUndo,
    NoRounds,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GameError::NeedTwoPlayers => "Ajoute au moins deux joueurs.",
            GameError::TooManyPlayers => "18 joueurs maximum.",
            GameError::EmptyName => "Indique un nom.",
            GameError::DuplicateName => "Ce joueur est déjà là.",
            GameError::NameTooLong => "24 caractères maximum pour un nom.",
            GameError::BadTarget => "L'objectif doit être un nombre entre 1 et 9999.",
            GameError::MissingScore => "Indique le score de chaque joueur. Mets 0 pour un éliminé.",
            GameError::BadScore => "Chaque score doit être un nombre entier.",
            GameError::NegativeScore => "Un score ne peut pas être négatif.",
            GameError::ScoreTooHigh => "Un score de manche ne peut pas dépasser 999.",
            GameError::ScoreCount => "Il manque le score d'un joueur.",
            GameError::NothingToUndo => "Aucune manche à annuler.",
            GameError::NoRounds => "Joue au moins une manche avant de terminer.",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub rank: usize,
    pub name: String,
    pub total: i32,
    pub last: Option<i32>,
    pub at_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub players: Vec<String>,
    pub rounds: Vec<Vec<i32>>,
    pub target: i32,
}

impl Game {
    pub fn new(players: Vec<String>, target: i32) -> Result<Self, GameError> {
        if !(1..=MAX_TARGET).contains(&target) {
            return Err(GameError::BadTarget);
        }
        if players.len() < MIN_PLAYERS {
            return Err(GameError::NeedTwoPlayers);
        }
        if players.len() > MAX_PLAYERS {
            return Err(GameError::TooManyPlayers);
        }

        let mut cleaned = Vec::with_capacity(players.len());
        for name in players {
            let name = clean_name(&name)?;
            if cleaned
                .iter()
                .any(|existing: &String| same_name(existing, &name))
            {
                return Err(GameError::DuplicateName);
            }
            cleaned.push(name);
        }

        Ok(Self {
            players: cleaned,
            rounds: Vec::new(),
            target,
        })
    }

    pub fn totals(&self) -> Vec<i32> {
        let mut totals: Vec<i32> = vec![0; self.players.len()];
        for round in &self.rounds {
            for (index, score) in round.iter().enumerate() {
                if let Some(total) = totals.get_mut(index) {
                    *total = (*total).saturating_add(*score);
                }
            }
        }
        totals
    }

    pub fn add_round(&mut self, scores: Vec<i32>) -> Result<(), GameError> {
        if scores.len() != self.players.len() {
            return Err(GameError::ScoreCount);
        }
        for score in &scores {
            if *score < 0 {
                return Err(GameError::NegativeScore);
            }
            if *score > MAX_ROUND_SCORE {
                return Err(GameError::ScoreTooHigh);
            }
        }
        self.rounds.push(scores);
        Ok(())
    }

    pub fn undo(&mut self) -> Result<Vec<i32>, GameError> {
        self.rounds.pop().ok_or(GameError::NothingToUndo)
    }

    pub fn target_reached(&self) -> bool {
        self.totals().iter().any(|total| *total >= self.target)
    }

    pub fn standings(&self) -> Vec<Standing> {
        let totals = self.totals();
        let last = self.rounds.last();
        let mut rows: Vec<Standing> = self
            .players
            .iter()
            .enumerate()
            .map(|(index, name)| Standing {
                rank: 0,
                name: name.clone(),
                total: totals.get(index).copied().unwrap_or(0),
                last: last.and_then(|round| round.get(index).copied()),
                at_target: totals.get(index).copied().unwrap_or(0) >= self.target,
            })
            .collect();

        rows.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));
        let mut previous_total: Option<i32> = None;
        let mut previous_rank = 0;
        for (index, row) in rows.iter_mut().enumerate() {
            if previous_total == Some(row.total) {
                row.rank = previous_rank;
            } else {
                row.rank = index + 1;
                previous_rank = row.rank;
            }
            previous_total = Some(row.total);
        }
        rows
    }

    pub fn winners(&self) -> Vec<String> {
        let standings = self.standings();
        let Some(best) = standings.first().map(|row| row.total) else {
            return Vec::new();
        };
        standings
            .into_iter()
            .filter(|row| row.total == best)
            .map(|row| row.name)
            .collect()
    }
}

pub fn clean_name(raw: &str) -> Result<String, GameError> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(GameError::EmptyName);
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(GameError::NameTooLong);
    }
    Ok(name.to_string())
}

pub fn same_name(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

pub fn parse_scores(draft: &[String]) -> Result<Vec<i32>, GameError> {
    draft
        .iter()
        .map(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(GameError::MissingScore);
            }
            let score: i32 = raw.parse().map_err(|_| GameError::BadScore)?;
            if score < 0 {
                return Err(GameError::NegativeScore);
            }
            if score > MAX_ROUND_SCORE {
                return Err(GameError::ScoreTooHigh);
            }
            Ok(score)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(names: &[&str]) -> Game {
        Game::new(names.iter().map(|name| (*name).to_string()).collect(), 200).unwrap()
    }

    #[test]
    fn rejects_a_single_player() {
        let error = Game::new(vec!["Camille".into()], 200).unwrap_err();
        assert_eq!(error, GameError::NeedTwoPlayers);
    }

    #[test]
    fn rejects_duplicate_and_blank_names() {
        assert_eq!(
            Game::new(vec!["Camille".into(), " camille ".into()], 200).unwrap_err(),
            GameError::DuplicateName
        );
        assert_eq!(
            Game::new(vec!["Camille".into(), "   ".into()], 200).unwrap_err(),
            GameError::EmptyName
        );
    }

    #[test]
    fn round_scores_accumulate_and_zero_stays_zero() {
        let mut game = game(&["Camille", "Samir"]);
        game.add_round(vec![24, 0]).unwrap();
        game.add_round(vec![10, 15]).unwrap();
        assert_eq!(game.totals(), vec![34, 15]);
    }

    #[test]
    fn undo_removes_only_the_last_round() {
        let mut game = game(&["Camille", "Samir"]);
        game.add_round(vec![24, 8]).unwrap();
        game.add_round(vec![10, 0]).unwrap();
        assert_eq!(game.undo().unwrap(), vec![10, 0]);
        assert_eq!(game.totals(), vec![24, 8]);
        game.undo().unwrap();
        assert_eq!(game.undo(), Err(GameError::NothingToUndo));
    }

    #[test]
    fn highest_score_wins_and_ties_share_the_rank() {
        let mut game = game(&["Camille", "Samir", "Lina"]);
        game.add_round(vec![40, 40, 10]).unwrap();
        assert_eq!(
            game.winners(),
            vec!["Camille".to_string(), "Samir".to_string()]
        );
        let standings = game.standings();
        assert_eq!(standings[0].rank, 1);
        assert_eq!(standings[1].rank, 1);
        assert_eq!(standings[2].rank, 3);
        assert!(!game.target_reached());
        game.add_round(vec![160, 0, 0]).unwrap();
        assert!(game.target_reached());
        assert_eq!(game.winners(), vec!["Camille".to_string()]);
    }

    #[test]
    fn rejects_invalid_scores() {
        let mut game = game(&["Camille", "Samir"]);
        assert_eq!(game.add_round(vec![1]).unwrap_err(), GameError::ScoreCount);
        assert_eq!(
            game.add_round(vec![-1, 0]).unwrap_err(),
            GameError::NegativeScore
        );
        assert_eq!(
            game.add_round(vec![1000, 0]).unwrap_err(),
            GameError::ScoreTooHigh
        );
        assert_eq!(
            parse_scores(&[String::new(), "4".into()]).unwrap_err(),
            GameError::MissingScore
        );
        assert_eq!(parse_scores(&["08".into()]).unwrap(), vec![8]);
    }
}
