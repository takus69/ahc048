use proconio::input;
use std::fmt;
use itertools::Itertools;
use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::time::{Duration, Instant};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;

#[derive(Debug, Clone)]
struct OrdFloat(f64);

impl PartialEq for OrdFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrdFloat {}

impl PartialOrd for OrdFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrdFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap() // NaNに注意！
    }
}

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
        Self { s: usize::MAX, e: 0.0, v: 0, h, d, cnt: 0 }
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

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Score: {}, d: {}, e: {}", self.s, self.score_d(), self.score_e())
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

#[derive(Debug, Clone, Copy)]
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

    fn add_color(&mut self, color: Color, r: f64) {
        self.c += color.c * r;
        self.m += color.m * r;
        self.y += color.y * r;
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

#[derive(Clone)]
struct Palette3 {
    own: Vec<Color>,
    color: Color,
    gram: f64,
    coef: Vec<f64>,
}

impl Palette3 {
    fn new(own: &Vec<Color>) -> Self {
        let own = own.clone();
        let color = Color::new(0.0, 0.0, 0.0);
        let gram = 0.0;
        let coef = vec![0.0; own.len()];

        Self { own, color, gram, coef }
    }
    
    fn init(&self, n: usize) -> Palette {
        let v = vec![vec![false; n-1]; n];
        let h = vec![vec![false; n]; n-1];

        Palette { v, h }
    }

    fn add_paint(&mut self, k: usize) {
        self.color = Color::mixing(self.color, self.own[k], self.gram as usize, 1);
        self.gram += 1.0;
        self.coef[k] += 1.0;
    }

    fn add_paints(&mut self, paints: &Vec<usize>) -> (Color, usize, usize, Vec<Action>) {
        let mut turn = 1;
        let mut cnt = 0;
        let mut actions: Vec<Action> = Vec::new();
        for (k, &c) in paints.iter().enumerate() {
            for _ in 0..c {
                self.add_paint(k);
                actions.push(Action::add_color(0, 0, k));
                cnt += 1;
                turn += 1;
            }
        }
        actions.push(Action::give_paint(0, 0));
        self.give_paint();

        (self.color, cnt, turn, actions)
    }

    fn give_paint(&mut self) -> Color {
        for i in 0..self.coef.len() {
            self.coef[i] -= self.coef[i] / self.gram;
        }
        self.gram -= 1.0;

        self.color
    }
}

#[derive(Clone)]
struct Palette4 {
    color: Vec<Color>,
    gram: Vec<usize>,
    comb: Vec<Vec<usize>>,
    comb2i: HashMap<usize, usize>,
}

impl Palette4 {
    fn new(own: &Vec<Color>, comb: &Vec<Vec<usize>>, comb_i: &Vec<usize>) -> Self {
        let comb = comb.clone();
        let mut color: Vec<Color> = Vec::new();
        let mut comb2i: HashMap<usize, usize> = HashMap::new();
        for i in 0..comb.len() {
            let comb2 = &comb[i];
            comb2i.insert(comb_i[i], i);
            let (mut c, mut m, mut y) = (0.0, 0.0, 0.0);
            for &k in comb2.iter() {
                c += own[k].c;
                m += own[k].m;
                y += own[k].y;
            }
            let cnt = comb2.len() as f64;
            c /= cnt;
            m /= cnt;
            y /= cnt;
            color.push(Color { c, m, y });
        }
        let gram: Vec<usize> = vec![0; comb.len()];

        Self { color, gram, comb, comb2i }
    }
    
    fn init(&self, n: usize) -> Palette {
        let v = vec![vec![false; n-1]; n];
        let h = vec![vec![true; n]; n-1];

        Palette { v, h }
    }

    fn give_paint(&mut self, comb_i: usize) -> (Color, usize, usize, Vec<Action>) {
        let mut cnt = 0;
        let mut turn = 0;
        let &i = self.comb2i.get(&comb_i).unwrap();
        let mut actions: Vec<Action> = Vec::new();
        if self.gram[i] == 0 {
            for &k in self.comb[i].iter() {
                actions.push(Action::add_color(i, 0, k));
                cnt += 1;
                turn += 1;
            } 
            self.gram[i] += self.comb[i].len();
        }
        actions.push(Action::give_paint(i, 0));
        self.gram[i] -= 1;
        turn += 1;

        (self.color[i], cnt, turn, actions)
    }
}

#[derive(Clone)]
struct Palette5 {
    base_color: Vec<Color>,
    base_gram: Vec<f64>,
    well_size: usize,
}

impl Palette5 {
    fn new(own: &Vec<Color>, well_size: usize) -> Self {
        let k = own.len();
        let mut base_color: Vec<Color> = Vec::new();
        for &o in own.iter() {
            base_color.push(o);
        }
        let base_gram: Vec<f64> = vec![0.0; k];

        Self { base_color, base_gram, well_size }
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        for i in 0..n {
            v[i][self.well_size-1] = true;
        }
        let mut h = vec![vec![false; n]; n-1];
        for i in 0..(n-1) {
            for j in 0..self.well_size {
                h[i][j] = true;
            }
        }

        Palette { v, h }
    }

    fn make_paint(&mut self, k1: usize, k2: usize, k3: usize, a: f64, b: f64) -> (Color, usize, usize, Vec<Action>) {
        let mut cnt = 0;
        let mut turn = 0;
        let mut actions: Vec<Action> = Vec::new();
        let well_size= self.well_size;

        // 係数は10倍して切り上げる
        let a = [((1.0-a-b)*well_size as f64), (a*well_size as f64), (b*well_size as f64)];
        let mut partition: Vec<(usize, usize)> = Vec::new();
        let mut use_grams: Vec<f64> = Vec::new();

        for (i, &k) in [k1, k2, k3].iter().enumerate() {
            // 絵の具の量が足りない場合は追加
            if self.base_gram[k] < a[i]/10.0 {
                actions.push(Action::add_color(k, 0, k));
                cnt += 1;
                turn += 1;
                self.base_gram[k] += 1.0;
            }
            // 必要な絵の具の量にするため、仕切りを追加
            let j = (a[i]/self.base_gram[k]).ceil() as usize;
            if j < well_size as usize {
                actions.push(Action::toggle_separator(k, well_size-1-j, k, well_size-j));  // 必要な割合に分ける
                partition.push((k, well_size-1-j));
                turn += 1;
            }
            actions.push(Action::toggle_separator(k, well_size-1, k, well_size));  // 混ぜる場所に繋げる
            turn += 1;
            let use_gram = self.base_gram[k] * j as f64 / well_size as f64;
            self.base_gram[k] -= use_gram;  // 残る絵の具の量を計算
            use_grams.push(use_gram);
        }

        // 絵の具の提出、残りの破棄
        actions.push(Action::give_paint(0, well_size));
        actions.push(Action::discard_paint(0, well_size));
        turn += 2;

        // 仕切りを元に戻す
        for (i, &k) in [k1, k2, k3].iter().enumerate() {
            actions.push(Action::toggle_separator(k, well_size-1, k, well_size));  // 混ぜる場所から隔離
            turn += 1;
        }
        for &(k, j) in partition.iter() {
            actions.push(Action::toggle_separator(k, j, k, j+1));  // 各絵の具の場所を元に戻す
            turn += 1;
        }

        // 作成した色を再現
        let mut color = Color::new(0.0, 0.0, 0.0);
        for (i, &k) in [k1, k2, k3].iter().enumerate() {
            color.add_color(self.base_color[k], use_grams[i]);
        }
        let all_gram: f64 = use_grams.iter().sum();
        color.c /= all_gram;
        color.m /= all_gram;
        color.y /= all_gram;

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
    timer: Instant,
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
        let timer = Instant::now();

        Self { n, k, h, t, d, own, target, palette, actions, turn, score, timer }
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

    fn base_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        // 絵の具の組み合わせ
        let elements = vec![0.0, 1.0];
        let mut comb: Vec<Vec<f64>> = vec![vec![]];
        for _ in 0..self.k {
            let mut tmp_comb: Vec<Vec<f64>> = Vec::new();
            for v in comb.iter() {
                for &e in elements.iter() {
                    let mut v2 = v.clone();
                    v2.push(e);
                    tmp_comb.push(v2);
                }
            }
            comb = tmp_comb;
        }

        fn mixing_color(own: &Vec<Color>, trial: &Vec<f64>) -> Color {
            let k = own.len();
            let (mut c, mut m, mut y) = (0.0, 0.0, 0.0);
            let mut all_cnt = 0.0;
            for i in 0..k {
                let color = own[i];
                let (ci, mi, yi) = (color.c, color.m, color.y);
                let cnt = trial[i];
                c += ci*cnt;
                m += mi*cnt;
                y += yi*cnt;
                all_cnt += cnt;
            }

            c /= all_cnt;
            m /= all_cnt;
            y /= all_cnt;

            Color { c, m, y }
        }

        let mut target_coef: Vec<Vec<f64>> = Vec::new();
        for &t in self.target.iter() {
            let mut opt_eval = f64::MAX;
            let mut opt_trial = vec![0.0; self.k];
            for trial in comb.iter() {
                let color = mixing_color(&self.own, trial);
                let e = t.dist(&color);
                if e < opt_eval {
                    opt_eval = e;
                    opt_trial = trial.clone();
                }
            }
            target_coef.push(opt_trial);
        }

        fn diff_coef(t_coef: &Vec<f64>, p_coef: &Vec<f64>) -> usize {
            let t_gram: f64 = t_coef.iter().sum();
            let p_gram: f64 = p_coef.iter().sum();

            let mut max_i = 0;
            let mut max_diff = 0.0;
            for (i, &ti) in t_coef.iter().enumerate() {
                let pi = p_coef[i];
                if ti*p_gram > pi*t_gram && max_diff < ti*p_gram - pi*t_gram {
                    max_diff = ti*p_gram - pi*t_gram;
                    max_i = i;
                }
            }
            
            max_i
        }

        let mut palette = Palette3::new(&self.own);
        let mut add_coef: Vec<Vec<usize>> = Vec::new();
        for i in 0..self.h {
            let t_coef = &target_coef[i];
            let target_color = self.target[i];

            let mut opt_eval = f64::MAX;
            let mut opt_coef = vec![0; self.k];
            let mut opt_palette = palette.clone();

            let mut loop_cnt = 0;
            loop {
                loop_cnt += 1;
                let max_diff_i = diff_coef(t_coef, &palette.coef);
                palette.add_paint(max_diff_i);
                let e = target_color.dist(&palette.color);
                let eval = e*10000.0 + (self.d*(loop_cnt-1)) as f64 / if self.h-i > palette.gram as usize { (self.h - i - palette.gram as usize) as f64 } else { 1.0 };

                if eval < opt_eval {
                    opt_eval = eval;
                    opt_coef[max_diff_i] += 1;
                    opt_palette = palette.clone();
                } else {
                    palette = opt_palette;
                    break;
                }
            }
            add_coef.push(opt_coef);
        }

        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        let mut palette = Palette3::new(&self.own);
        for i in 0..self.h {
            let paints= &add_coef[i];
            let target_color = self.target[i];
            let (color, cnt, t, action) = palette.add_paints(paints);
            score.add_score(target_color, color, cnt);
            turn += t;
            actions.extend(action);
        }

        (palette.init(self.n), actions, score, turn)
    }

    fn base_paint2(&self) -> (Palette, Vec<Action>, Score, usize) {
        // 絵の具の組み合わせ
        let mut comb: Vec<Vec<usize>> = Vec::new();
        let mut comb_i: Vec<usize> = Vec::new();
        let limit = 2_usize.pow(self.k as u32);
        let max_cnt = (limit-1).min(1000);
        // ランダムに重複なしの値を100個生成
        let seed = [0; 32]; // 固定のシード値を設定
        let mut rng = StdRng::from_seed(seed);
        let mut unique_values = HashSet::new();
        while unique_values.len() < max_cnt {
            let value = rng.gen_range(1..limit);
            unique_values.insert(value);
        }
        for i in unique_values {
            comb_i.push(i);
            let mut i = i;
            let mut cnt = 0;
            let mut tmp_comb: Vec<usize> = Vec::new();
            while i > 0 {
                if i % 2 == 1 {
                    tmp_comb.push(cnt);
                }
                i >>= 1;
                cnt += 1;
            }
            comb.push(tmp_comb);
        }

        // 目標の色のeが一番小さい組合せを探す
        fn mixing_color(own: &Vec<Color>, trial: &Vec<usize>) -> Color {
            let (mut c, mut m, mut y) = (0.0, 0.0, 0.0);
            for &k in trial.iter() {
                let color = own[k];
                let (ci, mi, yi) = (color.c, color.m, color.y);
                c += ci;
                m += mi;
                y += yi;
            }

            let cnt = trial.len() as f64;
            c /= cnt;
            m /= cnt;
            y /= cnt;

            Color { c, m, y }
        }

        let mut trials_heap: Vec<BinaryHeap<(Reverse<OrdFloat>, usize)>> = vec![BinaryHeap::new(); self.h];
        for (i, trial) in comb.iter().enumerate() {
            let p_color = mixing_color(&self.own, trial);
            for (j, t_color) in self.target.iter().enumerate() {
                let e = p_color.dist(t_color);
                trials_heap[j].push((Reverse(OrdFloat(e)), i));
            }
        }

        let mut trials_cnt: Vec<usize> = vec![0; comb.len()];
        let mut trials_e: Vec<f64> = vec![0.0; comb.len()];
        let mut estimate_e = 0.0;
        let mut target_i: Vec<Vec<usize>> = vec![Vec::new(); comb.len()];
        let mut out_of_scope: HashSet<usize> = HashSet::new();
        let mut update_target: Vec<usize> = (0..self.h).collect();
        let mut target_comb: Vec<usize> = vec![0; self.h];
        while update_target.len() > 20 {
            for &i in update_target.iter() {
                // 誤差が少ないパターンの上位の件数を取得
                let (Reverse(OrdFloat(mut e)), mut j) = trials_heap[i].pop().unwrap();
                while out_of_scope.contains(&j) {
                    (Reverse(OrdFloat(e)), j) = trials_heap[i].pop().unwrap();
                }
                target_i[j].push(i);
                trials_cnt[j] += 1;
                trials_e[j] += e;
                estimate_e += e;
                target_comb[i] = comb_i[j];
            }
            let mut heap_cnt: BinaryHeap<(Reverse<usize>, usize)> = trials_cnt.iter().enumerate().map(|(i, &cnt)| (Reverse(cnt), i)).collect();
            let pop_cnt = if heap_cnt.len() > 20 { heap_cnt.len() - 20 } else { 0 };
            update_target = Vec::new();
            for _ in 0..pop_cnt {
                let (_, j) = heap_cnt.pop().unwrap();
                out_of_scope.insert(j);
                for &l in target_i[j].iter() {
                    update_target.push(l);
                }
                target_i[j] = Vec::new();
                trials_cnt[j] = 0;
                estimate_e -= trials_e[j];
                trials_e[j] = 0.0;
            }
        }
        eprintln!("estimate_e: {:?}", estimate_e);

        // paletteの構築
        let mut comb2: Vec<Vec<usize>> = Vec::new();
        let mut comb_i2: Vec<usize> = Vec::new();
        for (i, &cnt) in trials_cnt.iter().enumerate() {
            if cnt > 0 {
                comb2.push(comb[i].clone());
                comb_i2.push(comb_i[i]);
            }
        }
        let mut palette = Palette4::new(&self.own, &comb2, &comb_i2);

        // Actionsの構築
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for i in 0..self.h { 
            let (color, cnt, t, action) = palette.give_paint(target_comb[i]);
            score.add_score(self.target[i], color, cnt);
            turn += t;
            actions.extend(action);
        }
        
        (palette.init(self.n), actions, score, turn)
    }

    fn three_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        // 関数定義
        fn dot(v1: Color, v2: Color) -> f64 {
            v1.c*v2.c + v1.m*v2.m + v1.y*v2.y
        }

        fn subtract(v1: Color, v2: Color) -> Color {
            Color { c: v1.c-v2.c, m: v1.m-v2.m, y: v1.y-v2.y }
        }

        fn mixing_3color(v1: Color, v2: Color, v3: Color, a: f64, b: f64) -> Color {
            let mut color = Color { c: 0.0, m: 0.0, y: 0.0 };
            color.add_color(v1, 1.0-a-b);
            color.add_color(v2, a);
            color.add_color(v3, b);

            color
        }

        fn opt_3color(t: Color, v1: Color, v2: Color, v3: Color) -> (Color, f64, f64) {
            let t_v1 = subtract(t, v1);
            let v3_v1 = subtract(v3, v1);
            let v2_v1 = subtract(v2, v1);

            let dot_v3v3 = dot(v3_v1, v3_v1);
            let dot_v2v2 = dot(v2_v1, v2_v1);
            let dot_v3v2 = dot(v3_v1, v2_v1);
            let dot_t_v3 = dot(t_v1, v3_v1);
            let dot_t_v2 = dot(t_v1, v2_v1);

            let denominator = dot_v3v3 * dot_v2v2 - dot_v3v2 * dot_v3v2;
            let b = (dot_t_v3 * dot_v2v2 - dot_t_v2 * dot_v3v2) / denominator;
            let a = (dot_t_v2 - b * dot_v3v2) / dot_v2v2;

            let color = mixing_3color(v1, v2, v3, a, b);
            (color, a, b)
        }

        // 各目標の色の3つの絵の具とその割合を算出
        let mut three_paint_params: Vec<(usize, usize, usize, f64, f64)> = Vec::new();
        let mut estimate_e = 0.0;

        for (i, &t) in self.target.iter().enumerate() {
            let mut opt_e = f64::MAX;
            let mut opt_params = (0, 0, 0, 0.0, 0.0);

            for k1 in 0..self.k {
                for k2 in (k1+1)..self.k {
                    for k3 in (k2+1)..self.k {
                        let v1 = self.own[k1];
                        let v2 = self.own[k2];
                        let v3 = self.own[k3];

                        let (color, a, b) = opt_3color(t, v1, v2, v3);
                        if !(0.0..1.0).contains(&(1.0-a-b)) || !(0.0..1.0).contains(&a) || !(0.0..1.0).contains(&b) { continue; }
                        let e = t.dist(&color);
                        if opt_e > e {
                            opt_e = e;
                            opt_params = (k1, k2, k3, a, b);
                        }
                    }
                }
            }
            three_paint_params.push(opt_params);
            estimate_e += opt_e;
            if i < 10 {
                eprintln!("i: {}, opt_e: {}", i, opt_e);
            }
        }
        eprintln!("estimate_e: {}", estimate_e);

        // Actionsの構築
        let mut palette = Palette5::new(&self.own, 19);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for i in 0..self.h { 
            let (k1, k2, k3, a, b) = three_paint_params[i];
            let (color, cnt, t, action) = palette.make_paint(k1, k2, k3, a, b);
            if i < 10 {
                eprintln!("i: {}, e: {}", i, color.dist(&self.target[i]));
            }
            score.add_score(self.target[i], color, cnt);
            turn += t;
            actions.extend(action);
        }

        (palette.init(self.n), actions, score, turn)
    }

    fn solve(&mut self) {
        eprintln!("start solve: {:.2?} s", self.timer.elapsed());
        (self.palette, self.actions, self.score, self.turn) = self.one_paint(); 
        eprintln!("{:.2?} s: one paint: {}", self.timer.elapsed(), self.score);
        
        let (palette, actions, score, turn) = self.two_paint();
        eprintln!("{:.2?} s: two paint: {}", self.timer.elapsed(), score);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn) = self.base_paint2();
        eprintln!("{:.2?} s: base paint2: {}", self.timer.elapsed(), score);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn) = self.three_paint();
        eprintln!("{:.2?} s: three paint: {}", self.timer.elapsed(), score);
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
