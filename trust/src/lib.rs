#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

pub struct Game {
    left: Box<dyn Agent>,
    right: Box<dyn Agent>,
    left_score: i32,
    right_score: i32,
}

impl Game {
    pub fn new(left: Box<dyn Agent>, right: Box<dyn Agent>) -> Self {
        Self {
            left,
            right,
            left_score: 0,
            right_score: 0,
        }
    }

    pub fn left_score(&self) -> i32 {
        self.left_score
    }

    pub fn right_score(&self) -> i32 {
        self.right_score
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let r1 = self.left.play();
        let r2 = self.right.play();
        self.left.opponents_move(r2);
        self.right.opponents_move(r1);
        if r1 {
            if r2 {
                self.change_score(2, 2);
                RoundOutcome::BothCooperated
            } else {
                self.change_score(-1, 3);
                RoundOutcome::RightCheated
            }
        } else if r2 {
            self.change_score(3, -1);
            RoundOutcome::LeftCheated
        } else {
            RoundOutcome::BothCheated
        }
    }

    fn change_score(&mut self, left_change: i32, right_change: i32) {
        self.left_score += left_change;
        self.right_score += right_change;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Agent {
    fn play(&mut self) -> bool;
    fn opponents_move(&mut self, opponents_move: bool);
}

pub struct CheatingAgent {}

impl Default for CheatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CheatingAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for CheatingAgent {
    fn play(&mut self) -> bool {
        false
    }

    fn opponents_move(&mut self, _: bool) {}
}

////////////////////////////////////////////////////////////////////////////////

pub struct CooperatingAgent {}

impl Default for CooperatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CooperatingAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for CooperatingAgent {
    fn play(&mut self) -> bool {
        true
    }

    fn opponents_move(&mut self, _: bool) {}
}

////////////////////////////////////////////////////////////////////////////////

pub struct GrudgerAgent {
    deceived: bool,
}

impl Default for GrudgerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl GrudgerAgent {
    pub fn new() -> Self {
        Self { deceived: false }
    }
}

impl Agent for GrudgerAgent {
    fn play(&mut self) -> bool {
        !self.deceived
    }

    fn opponents_move(&mut self, opponents_move: bool) {
        if !opponents_move {
            self.deceived = true;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CopycatAgent {
    next_move: bool,
}

impl Default for CopycatAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CopycatAgent {
    pub fn new() -> Self {
        Self { next_move: true }
    }
}

impl Agent for CopycatAgent {
    fn play(&mut self) -> bool {
        self.next_move
    }

    fn opponents_move(&mut self, opponents_move: bool) {
        self.next_move = opponents_move;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct DetectiveAgent {
    moves: Vec<bool>,
    deceived: bool,
    next_move: bool,
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectiveAgent {
    pub fn new() -> Self {
        Self {
            moves: vec![true, true, false, true],
            deceived: false,
            next_move: true,
        }
    }
}

impl Agent for DetectiveAgent {
    fn play(&mut self) -> bool {
        if self.moves.is_empty() {
            if self.deceived {
                self.next_move
            } else {
                false
            }
        } else {
            self.moves.pop().unwrap()
        }
    }

    fn opponents_move(&mut self, opponents_move: bool) {
        if !opponents_move {
            self.deceived = true;
        }
        self.next_move = opponents_move;
    }
}
