use serde::{Deserialize, Serialize};

use crate::game::{clean_name, parse_scores, same_name, Game, GameError, MAX_PLAYERS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Setup,
    Playing,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub phase: Phase,
    pub players: Vec<String>,
    pub rounds: Vec<Vec<i32>>,
    pub target: i32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            phase: Phase::Setup,
            players: Vec::new(),
            rounds: Vec::new(),
            target: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved {
    pub phase: Phase,
    pub players: Vec<String>,
    pub rounds: Vec<Vec<i32>>,
    pub target: i32,
    pub target_input: String,
    pub draft: Vec<String>,
}

impl Session {
    pub fn game(&self) -> Game {
        Game {
            players: self.players.clone(),
            rounds: self.rounds.clone(),
            target: self.target,
        }
    }

    pub fn add_player(&mut self, raw: &str) -> Result<(), GameError> {
        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::TooManyPlayers);
        }
        let name = clean_name(raw)?;
        if self.players.iter().any(|player| same_name(player, &name)) {
            return Err(GameError::DuplicateName);
        }
        self.players.push(name);
        Ok(())
    }

    pub fn remove_player(&mut self, name: &str) {
        if self.phase != Phase::Setup {
            return;
        }
        self.players.retain(|player| player != name);
    }

    pub fn start(&mut self, target_text: &str) -> Result<(), GameError> {
        let target = target_text
            .trim()
            .parse::<i32>()
            .map_err(|_| GameError::BadTarget)?;
        let game = Game::new(self.players.clone(), target)?;
        self.players = game.players;
        self.target = game.target;
        self.rounds.clear();
        self.phase = Phase::Playing;
        Ok(())
    }

    pub fn submit_round(&mut self, draft: &[String]) -> Result<(), GameError> {
        let scores = parse_scores(draft)?;
        let mut game = self.game();
        game.add_round(scores)?;
        self.rounds = game.rounds;
        Ok(())
    }

    pub fn undo(&mut self) -> Result<Vec<String>, GameError> {
        let scores = self.rounds.pop().ok_or(GameError::NothingToUndo)?;
        self.phase = Phase::Playing;
        Ok(scores.into_iter().map(|score| score.to_string()).collect())
    }

    pub fn finish(&mut self) -> Result<(), GameError> {
        if self.rounds.is_empty() {
            return Err(GameError::NoRounds);
        }
        self.phase = Phase::Finished;
        Ok(())
    }

    pub fn rematch(&mut self) {
        self.rounds.clear();
        self.phase = Phase::Playing;
    }

    pub fn to_saved(&self, target_input: &str, draft: &[String]) -> Saved {
        Saved {
            phase: self.phase,
            players: self.players.clone(),
            rounds: self.rounds.clone(),
            target: self.target,
            target_input: target_input.to_string(),
            draft: draft.to_vec(),
        }
    }

    pub fn from_saved(saved: Saved) -> (Self, Vec<String>, String) {
        let mut players = Vec::new();
        for name in saved.players {
            let Ok(name) = clean_name(&name) else {
                continue;
            };
            if players
                .iter()
                .any(|player: &String| same_name(player, &name))
            {
                continue;
            }
            players.push(name);
            if players.len() == MAX_PLAYERS {
                break;
            }
        }

        let target = if (1..=crate::game::MAX_TARGET).contains(&saved.target) {
            saved.target
        } else {
            200
        };

        let mut rounds = saved
            .rounds
            .into_iter()
            .filter(|round| {
                round.len() == players.len()
                    && round
                        .iter()
                        .all(|score| (0..=crate::game::MAX_ROUND_SCORE).contains(score))
            })
            .collect::<Vec<_>>();

        let mut phase = saved.phase;
        if players.len() < crate::game::MIN_PLAYERS {
            phase = Phase::Setup;
            rounds.clear();
        } else if phase == Phase::Finished && rounds.is_empty() {
            phase = Phase::Setup;
        }

        let mut draft = saved
            .draft
            .into_iter()
            .map(|value| {
                value
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .take(3)
                    .collect()
            })
            .collect::<Vec<String>>();
        draft.resize(players.len(), String::new());

        let target_input = {
            let digits: String = saved
                .target_input
                .chars()
                .filter(|c| c.is_ascii_digit())
                .take(4)
                .collect();
            if digits.is_empty() {
                target.to_string()
            } else {
                digits
            }
        };

        (
            Self {
                phase,
                players,
                rounds,
                target,
            },
            draft,
            target_input,
        )
    }
}

pub fn format_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} et {second}"),
        many => {
            let (last, rest) = many.split_last().expect("at least three names");
            format!("{} et {last}", rest.join(", "))
        }
    }
}

pub fn digits_only(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(max_chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_scoring_and_undo_restore_the_draft() {
        let mut session = Session::default();
        session.add_player(" Camille ").unwrap();
        assert_eq!(
            session.add_player("camille").unwrap_err(),
            GameError::DuplicateName
        );
        session.add_player("Samir").unwrap();
        session.start("200").unwrap();
        assert_eq!(session.phase, Phase::Playing);

        session.submit_round(&["24".into(), "0".into()]).unwrap();
        session.submit_round(&["10".into(), "15".into()]).unwrap();
        assert_eq!(session.game().totals(), vec![34, 15]);

        let restored = session.undo().unwrap();
        assert_eq!(restored, vec!["10".to_string(), "15".to_string()]);
        assert_eq!(session.game().totals(), vec![24, 0]);
        assert_eq!(session.phase, Phase::Playing);
    }

    #[test]
    fn finish_requires_a_round_and_names_a_winner() {
        let mut session = Session::default();
        session.add_player("Camille").unwrap();
        session.add_player("Samir").unwrap();
        session.start("50").unwrap();
        assert_eq!(session.finish().unwrap_err(), GameError::NoRounds);
        session.submit_round(&["40".into(), "60".into()]).unwrap();
        session.finish().unwrap();
        assert_eq!(session.phase, Phase::Finished);
        assert_eq!(session.game().winners(), vec!["Samir".to_string()]);
        assert!(session.game().target_reached());
    }

    #[test]
    fn corrupt_save_drops_bad_rounds_and_keeps_players() {
        let saved = Saved {
            phase: Phase::Playing,
            players: vec![
                "Camille".into(),
                "camille".into(),
                "Samir".into(),
                " ".into(),
            ],
            rounds: vec![vec![10, 0, 5], vec![1, 2]],
            target: 0,
            target_input: "20a0".into(),
            draft: vec!["12x".into()],
        };
        let (session, draft, target_input) = Session::from_saved(saved);
        assert_eq!(
            session.players,
            vec!["Camille".to_string(), "Samir".to_string()]
        );
        assert_eq!(session.rounds, vec![vec![1, 2]]);
        assert_eq!(session.target, 200);
        assert_eq!(target_input, "200");
        assert_eq!(draft, vec!["12".to_string(), String::new()]);
    }

    #[test]
    fn format_names_uses_french_conjunction() {
        assert_eq!(format_names(&["A".into()]), "A");
        assert_eq!(format_names(&["A".into(), "B".into()]), "A et B");
        assert_eq!(
            format_names(&["A".into(), "B".into(), "C".into()]),
            "A, B et C"
        );
    }
}
