use proconio::input;
use std::fmt;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Score {
    s: usize,
    e: f64,
    v: usize,
    h: usize,
    d: usize,
    cnt: usize,
}

impl Score {
    fn new(h: usize, d: usize) -> Self {
        Self { s: 1, e: 0.0, v: 0, h, d, cnt: 0 }
    }

    fn dist(color1: Color, color2: Color) -> f64 {
        color1.dist(&color2)
    }

    fn eval(&mut self, t: Color, color: Color, cnt: usize) -> (usize, f64) {
        let e = Score::dist(t, color);
        let score = self.d*(cnt-1) + (e*10000.0).round() as usize;

        (score, e)
    }

    fn add_score(&mut self, t: Color, color: Color, cnt: usize) {
        self.cnt += 1;
        self.e += Score::dist(t, color);
        self.v += cnt;
        self.s = 1 + self.score_d() + self.score_e();
    }

    fn score_d(&self) -> usize {
        self.d*(self.v-self.cnt)
    }

    fn score_e(&self) -> usize {
        (self.e*10000.0).round() as usize
    }

    fn score(&self) -> (usize, usize, usize) {
        (self.s, self.score_d(), self.score_e())
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.s.partial_cmp(&other.s)
    }
}

#[derive(Clone, Copy)]
struct Color {
    c: f64,
    m: f64,
    y: f64,
}

impl Color {
    fn new(c: f64, m: f64, y: f64) -> Self {
        Self { c, m, y }
    }

    fn dist(&self, other: &Color) -> f64 {
        ((self.c - other.c).powi(2) 
        + (self.m - other.m).powi(2) 
        + (self.y - other.y).powi(2))
        .sqrt()
    }

    fn mixing(color1: Color, color2: Color, i1: usize, i2: usize) -> Self {
        let i1 = i1 as f64;
        let i2 = i2 as f64;
        let c = (color1.c*i1 + color2.c*i2) / (i1+i2);
        let m = (color1.m*i1 + color2.m*i2) / (i1+i2);
        let y = (color1.y*i1 + color2.y*i2) / (i1+i2);

        Self { c, m, y }
    }
}

#[derive(Debug)]
enum ActionType {
    AddPaint,        // 絵の具を追加
    GivePaint,       // 絵の具を渡す
    DiscardPaint,    // 絵の具を廃棄
    ToggleSeparator, // 仕切りを切り替え
}

#[derive(Debug)]
struct Action {
    action_type: ActionType, // 操作の種類
    i: usize,                // マスの座標i
    j: usize,                // マスの座標j
    k: Option<usize>,        // チューブ番号（必要な場合のみ）
    i2: Option<usize>,       // 隣接マスの座標i（仕切り操作用）
    j2: Option<usize>,       // 隣接マスの座標j（仕切り操作用）
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.action_type {
            ActionType::AddPaint => {
                write!(f, "1 {} {} {}", self.i, self.j, self.k.unwrap())
            },
            ActionType::GivePaint => {
                write!(f, "2 {} {}", self.i, self.j)
            },
            ActionType::DiscardPaint => {
                write!(f, "3 {} {}", self.i, self.j)
            },
            ActionType::ToggleSeparator => {
                write!(f, "4 {} {} {} {}", self.i, self.j, self.i2.unwrap(), self.j2.unwrap())
            },
        }
    }
}

impl Action {
    fn new(action_type: ActionType, i: usize, j: usize, k: Option<usize>, i2: Option<usize>, j2: Option<usize>) -> Self {
        Self {
            action_type,
            i,
            j,
            k,
            i2,
            j2,
        }
    }

    fn add_color(i: usize, j: usize, k: usize) -> Self {
        Action::new(ActionType::AddPaint, i, j, Some(k), None, None)
    }

    fn give_paint(i: usize, j: usize) -> Self {
        Action::new(ActionType::GivePaint, i, j, None, None, None)
    }
    
    fn discard_paint(i: usize, j: usize) -> Self {
        Action::new(ActionType::DiscardPaint, i, j, None, None, None)
    }

    fn toggle_separator(i: usize, j: usize, i2: usize, j2: usize) -> Self {
        Action::new(ActionType::ToggleSeparator, i, j, None, Some(i2), Some(j2))
    }
}

#[derive(Debug)]
struct Palette {
    v: Vec<Vec<bool>>,
    h: Vec<Vec<bool>>,
}

impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for vi in self.v.iter() {
            writeln!(f, "{}", vi.into_iter().map(|&b| if b { '1' } else { '0' }).collect::<Vec<_>>().iter().join(" "))?;
        }
        for hi in self.h.iter() {
            writeln!(f, "{}", hi.into_iter().map(|&b| if b { '1' } else { '0' }).collect::<Vec<_>>().iter().join(" "))?;
        }

        Ok(())
    }
}

struct Palette1 {
    own: Vec<Color>,
}

impl Palette1 {
    fn new(own: &Vec<Color>) -> Self {
        let own = own.clone();
        Self { own }
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        let mut h = vec![vec![false; n]; n-1];
        v[0][0] = true;
        h[0][0] = true;

        Palette { v, h }
    }

    fn make_color(&self, i: usize) -> (Color, usize, usize, Vec<Action>) {
        let mut actions: Vec<Action> = Vec::new();
        actions.push(Action::add_color(0, 0, i));
        actions.push(Action::give_paint(0, 0));
        let color = self.own[i];

        (color, 1, 2, actions)
    }
}

struct Palette2 {
    own: Vec<Color>,
    row: HashMap<usize, usize>,
    primes: Vec<usize>,
}

impl Palette2 {
    fn new(own: &Vec<Color>, primes: Vec<usize>) -> Self {
        let own = own.clone();
        let mut row: HashMap<usize, usize> = HashMap::new();
        for (i, &p) in primes.iter().enumerate() {
            row.insert(p, i);
        }
        Self { own, row, primes }
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        let h = vec![vec![true; n]; n-1];

        for (&p, &i) in self.row.iter() {
            v[i][p-1] = true;
        }

        Palette { v, h }
    }

    fn make_color(&self, p: usize, i: usize, k1: usize, k2: usize) -> (Color, usize, usize, Vec<Action>) {
        let mut actions: Vec<Action> = Vec::new();
        let &pi = self.row.get(&p).unwrap();
        let (cnt, turn, color) = if p == 1 {
            actions.push(Action::add_color(pi, 0, k1));
            actions.push(Action::give_paint(pi, 0));

            (1, 2, self.own[k1])
        } else {
            let mut tmp_turn = 4;
            actions.push(Action::add_color(pi, 0, k2));
            if p != i {
                actions.push(Action::toggle_separator(pi, i-1, pi, i));
                tmp_turn += 1;
            }
            actions.push(Action::add_color(pi, 0, k1));
            actions.push(Action::give_paint(pi, 0));
            if p != i {
                actions.push(Action::toggle_separator(pi, i-1, pi, i));
                tmp_turn += 1;
            }
            actions.push(Action::discard_paint(pi, 0));
            let color = if i == 1 {
                Color::mixing(self.own[k1], self.own[k2], p-1, i)
            } else {
                Color::mixing(self.own[k1], self.own[k2], p, i)
            };

            (2, tmp_turn, color)
        };

        (color, cnt, turn, actions)
    }
}

struct Solver {
    n: usize,
    k: usize,
    h: usize,
    t: usize,
    d: usize,
    own: Vec<Color>,
    target: Vec<Color>,
    palette: Palette,
    actions: Vec<Action>,
    turn: usize,
    score: Score,
}

impl Solver {
    fn new() -> Self {
        input! {
            n: usize,
            k: usize,
            h: usize,
            t: usize,
            d: usize,
            own: [(f64, f64, f64); k],
            target: [(f64, f64, f64); h],
        }

        let mut own2: Vec<Color> = Vec::new();
        for &(c, m, y) in own.iter() {
            own2.push(Color::new(c, m, y));
        }
        let own = own2;

        let mut target2: Vec<Color> = Vec::new();
        for &(c, m, y) in target.iter() {
            target2.push(Color::new(c, m, y));
        }
        let target= target2;

        let palette = Palette{ v: Vec::new(), h: Vec::new()};
        let actions: Vec<Action> = Vec::new();
        let turn = 0;
        let score = Score::new(h, d);

        Self { n, k, h, t, d, own, target, palette, actions, turn, score }
    }

    fn one_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        // 1色の最適な絵の具を選択
        let palette = Palette1::new(&self.own);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for &target_color in self.target.iter() {
            let mut opt_eval= usize::MAX;
            let mut opt_action: Vec<Action> = Vec::new();
            let mut opt_score = score;
            let mut opt_turn = 0;
            for i in 0..self.k {
                let (color, cnt, t, action) = palette.make_color(i);
                let (eval, _) = score.eval(target_color, color, cnt);
                if eval < opt_eval {
                    opt_eval = eval;
                    opt_action = action;
                    opt_score = score;
                    opt_score.add_score(target_color, color, cnt);
                    opt_turn = t;
                }
            }
            actions.extend(opt_action);
            score = opt_score;
            turn += opt_turn;
        }

        (palette.init(self.n), actions, score, turn)
    }

    fn two_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        let primes = vec![1, 2, 3, 5];
        let palette = Palette2::new(&self.own, primes);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for &target_color in self.target.iter() {
            let mut opt_eval= usize::MAX;
            let mut opt_action: Vec<Action> = Vec::new();
            let mut opt_score = score;
            let mut opt_turn = 0;
            for k1 in 0..self.k {
                for k2 in 0..self.k {
                    for &p in palette.primes.iter() {
                        let end = if p <= 2 { p+1 } else { p };
                        for i in 1..end {
                            let (color, cnt, t, action) = palette.make_color(p, i, k1, k2);
                            let (eval, _) = score.eval(target_color, color, cnt);
                            if eval < opt_eval {
                                opt_eval = eval;
                                opt_action = action;
                                opt_score = score;
                                opt_score.add_score(target_color, color, cnt);
                                opt_turn = t;
                            }
                        }
                    }
                }
            }
            actions.extend(opt_action);
            score = opt_score;
            turn += opt_turn;
        }

        (palette.init(self.n), actions, score, turn)
    }

    fn solve(&mut self) {
        (self.palette, self.actions, self.score, self.turn) = self.one_paint(); 
        
        let (palette, actions, score, turn) = self.two_paint();
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }
    }

    fn ans(&self) {
        print!("{}", self.palette);
        for action in self.actions.iter() {
            println!("{}", action);
        }
    }

    fn result(&self) {
        let (score, score_d, score_e) = self.score.score();
        eprintln!("{{ \"n\": {}, \"k\": {}, \"h\": {}, \"max_turn\": {}, \"turn\": {}, \"d\": {}, \"score\": {}, \"score_d\": {}, \"score_e\": {}, \"v\": {}, \"e\": {} }}", self.n, self.k, self.h, self.t, self.turn, self.d, score, score_d, score_e, self.score.v, self.score.e);
    }
}

fn main() {
    let mut solver = Solver::new();
    solver.solve();
    solver.ans();
    solver.result();
}
