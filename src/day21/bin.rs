use std::collections::HashMap;

use itertools::Itertools;

pub trait Die {
    fn roll(&mut self) -> u32;
}

pub trait QuantumDie {
    fn roll(&self) -> &[u32];
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pos: u8,
    score: u32,
}

impl Player {
    pub fn new(pos: u8) -> Self {
        assert!(
            1 < pos && pos <= 10,
            "pos must be in 1..=10, but was {}",
            pos
        );
        Self {
            pos: pos - 1,
            score: 0,
        }
    }

    pub fn pos(&self) -> u8 {
        self.pos + 1
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn advance(&mut self, distance: u32) {
        self.pos = ((self.pos as u32 + distance) % 10) as u8;
        self.score += self.pos() as u32;
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Game {
    p1: Player,
    p2: Player,
}

impl Game {
    pub fn new_game(p1_pos: u8, p2_pos: u8) -> Self {
        Self {
            p1: Player::new(p1_pos),
            p2: Player::new(p2_pos),
        }
    }

    #[must_use]
    pub fn play<D: Die>(mut self, die: &mut D, win_score: u32) -> WinState {
        let mut num_rolls = 0;
        let winner_is_p1 = 'all_rounds: loop {
            for is_p1 in [true, false] {
                let roll_sum = die.roll() + die.roll() + die.roll();
                num_rolls += 3;
                let curr = if is_p1 { &mut self.p1 } else { &mut self.p2 };
                curr.advance(roll_sum);
                if curr.score() >= win_score {
                    break 'all_rounds is_p1;
                }
            }
        };
        WinState {
            winner_is_p1,
            p1: self.p1,
            p2: self.p2,
            num_rolls,
        }
    }

    #[must_use]
    pub fn play_quantum<D: QuantumDie>(self, die: &D, win_score: u32) -> QuantumWinState {
        let mut cached_wins = HashMap::new();

        let mut roll_sums = HashMap::new();
        for roll1 in die.roll() {
            for roll2 in die.roll() {
                for roll3 in die.roll() {
                    *roll_sums.entry(roll1 + roll2 + roll3).or_default() += 1;
                }
            }
        }
        let roll_sums: Vec<_> = roll_sums.into_iter().collect();

        fn play_verse(
            game: Game,
            roll_sums: &[(u32, u64)],
            win_score: u32,
            cached_wins: &mut HashMap<Game, QuantumWinState>,
        ) -> QuantumWinState {
            if let Some(cached_wins) = cached_wins.get(&game) {
                cached_wins.clone()
            } else {
                let mut p1_wins = 0;
                let mut p2_wins = 0;
                for (roll, roll_count) in roll_sums {
                    let mut after_p1 = game.clone();
                    after_p1.p1.advance(*roll);
                    if after_p1.p1.score() >= win_score {
                        p1_wins += roll_count;
                    } else {
                        for (roll, roll2_count) in roll_sums {
                            let roll_count = roll_count * roll2_count;
                            let mut after_p2 = after_p1.clone();
                            after_p2.p2.advance(*roll);
                            if after_p2.p2.score() >= win_score {
                                p2_wins += roll_count;
                            } else {
                                let rest = play_verse(after_p2, roll_sums, win_score, cached_wins);
                                p1_wins += rest.p1_wins * roll_count;
                                p2_wins += rest.p2_wins * roll_count;
                            }
                        }
                    }
                }
                let r = QuantumWinState { p1_wins, p2_wins };
                cached_wins.insert(game, r.clone());
                r
            }
        }

        play_verse(self, roll_sums.as_slice(), win_score, &mut cached_wins)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WinState {
    pub winner_is_p1: bool,
    pub p1: Player,
    pub p2: Player,
    pub num_rolls: u32,
}

impl WinState {
    pub fn winner(&self) -> &Player {
        if self.winner_is_p1 {
            &self.p1
        } else {
            &self.p2
        }
    }

    pub fn loser(&self) -> &Player {
        if self.winner_is_p1 {
            &self.p2
        } else {
            &self.p1
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QuantumWinState {
    pub p1_wins: u64,
    pub p2_wins: u64,
}

impl QuantumWinState {
    pub fn min_max_wins(&self) -> (u64, u64) {
        if self.p1_wins > self.p2_wins {
            (self.p2_wins, self.p1_wins)
        } else {
            (self.p1_wins, self.p2_wins)
        }
    }
}

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq)]
pub struct DeterministicD100 {
    next: u8,
}

impl Die for DeterministicD100 {
    fn roll(&mut self) -> u32 {
        let r = self.next as u32 + 1;
        self.next = (self.next + 1) % 100;
        r
    }
}

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq)]
pub struct DiracDie;

impl QuantumDie for DiracDie {
    fn roll(&self) -> &[u32] {
        &[1, 2, 3]
    }
}

fn main() {
    let (p1, p2) = include_str!("input.txt")
        .lines()
        .map(|t| {
            let (_, pos) = t.split_once(" starting position: ").unwrap();
            pos.parse().unwrap()
        })
        .collect_tuple()
        .unwrap();
    let game = Game::new_game(p1, p2);

    let mut die = DeterministicD100::default();
    let win_state = game.clone().play(&mut die, 1000);
    let loser_score = win_state.loser().score();
    println!(
        "P1: {} * {} = {}",
        loser_score,
        win_state.num_rolls,
        loser_score * win_state.num_rolls
    );

    let die = DiracDie::default();
    let win_state = game.play_quantum(&die, 21);
    let (min_wins, max_wins) = win_state.min_max_wins();
    println!("P2: quantum wins: {} >= {}", max_wins, min_wins);
}
