use proconio::input;
use std::fmt;
use itertools::{enumerate, Itertools};
use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::time::{Duration, Instant};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use nalgebra::Matrix3;

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

struct Palette2 {
    own: Vec<Color>,
    row: HashMap<usize, usize>,
    primes: Vec<usize>,
}

impl Palette2 {
    fn new(own: &Vec<Color>, primes: Vec<usize>) -> Self {
        let own = own.clone();
        let mut row: HashMap<usize, usize> = HashMap::new();
        let mut i = 0;
        for &p in primes.iter() {
            row.insert(p, i);
            i += p;
        }
        Self { own, row, primes }
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        let mut h = vec![vec![false; n]; n-1];

        for i in 0..n {
            v[i][18] = true;
        }

        for (&p, &i) in self.row.iter() {
            h[i+p-1][19] = true;
        }

        Palette { v, h }
    }

    fn make_color(&self, p: usize, i: usize, k1: usize, k2: usize) -> (Color, usize, usize, Vec<Action>) {
        let mut actions: Vec<Action> = Vec::new();
        let &pi = self.row.get(&p).unwrap();
        let (cnt, turn, color) = if p == 1 {
            actions.push(Action::add_color(pi, 19, k1));
            actions.push(Action::give_paint(pi, 19));

            (1, 2, self.own[k1])
        } else {
            let mut tmp_turn = 4;
            actions.push(Action::add_color(pi, 19, k2));
            if p != i {
                actions.push(Action::toggle_separator(pi+i-1, 19, pi+i, 19));
                tmp_turn += 1;
            }
            actions.push(Action::add_color(pi, 19, k1));
            actions.push(Action::give_paint(pi, 19));
            if p != i {
                actions.push(Action::toggle_separator(pi+i-1, 19, pi+i, 19));
                tmp_turn += 1;
            }
            actions.push(Action::discard_paint(pi, 19));
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

    fn init2(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        let mut h = vec![vec![false; n]; n-1];

        for i in 0..n {
            v[i][14] = true;
            v[i][17] = true;
            if i == 19 { continue; }
            for j in 15..=17 {
                h[i][j] = true;
            }
        }

        Palette { v, h }
    }

    fn give_paint(&mut self, comb_i: usize) -> (Color, usize, usize, Vec<Action>) {
        let mut cnt = 0;
        let mut turn = 0;
        let &i = self.comb2i.get(&comb_i).unwrap();
        let mut actions: Vec<Action> = Vec::new();
        if self.gram[i] == 0 {
            for &k in self.comb[i].iter() {
                actions.push(Action::add_color(i, 15, k));
                cnt += 1;
                turn += 1;
            } 
            self.gram[i] += self.comb[i].len();
        }
        actions.push(Action::give_paint(i, 15));
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
            v[i][self.well_size] = true;
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

        // 係数はwell_size倍する
        let a = [((1.0-a-b)*well_size as f64), (a*well_size as f64), (b*well_size as f64)];
        let mut partition: Vec<(usize, usize)> = Vec::new();
        let mut use_grams: Vec<f64> = Vec::new();
        let mut all_gram = 0.0;

        for (i, &k) in [k1, k2, k3].iter().enumerate() {
            // 絵の具の量が足りない場合は追加
            if self.base_gram[k] < a[i]/well_size as f64 {
                actions.push(Action::add_color(k, 0, k));
                cnt += 1;
                turn += 1;
                self.base_gram[k] += 1.0;
            }
            // 必要な絵の具の量にするため、仕切りを追加
            let mut j = (a[i]/self.base_gram[k]).floor() as usize;
            all_gram += self.base_gram[k] * j as f64 / well_size as f64;
            if all_gram < 1.0 { j += 1; }
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

#[derive(Clone)]
struct Palette6 {
    base_color: Vec<Color>,
    remain_color: Vec<Color>,
    remain_gram: Vec<f64>,
    start_j: usize,
    well_size: usize,
}

impl Palette6 {
    fn new(own: &Vec<Color>, start_j: usize, well_size: usize) -> Self {
        let n = 20;
        let mut base_color: Vec<Color> = Vec::new();
        for &o in own.iter() {
            base_color.push(o);
        }
        let remain_color: Vec<Color> = vec![Color { c: 0.0, m: 0.0, y: 0.0 }; n];
        let remain_gram: Vec<f64> = vec![0.0; n];

        Self { base_color, remain_color, remain_gram, start_j, well_size }
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        let end_j = self.start_j+self.well_size-1;
        for i in 0..n {
            if self.start_j > 0 {
                v[i][self.start_j-1] = true;
            }
            if end_j < 19 {
                v[i][end_j] = true;
            }
        }
        let mut h = vec![vec![false; n]; n-1];
        for i in 0..(n-1) {
            for j in self.start_j..=end_j {
                h[i][j] = true;
            }
        }

        Palette { v, h }
    }

    fn make_paint(&mut self, i: usize, k1: usize, k2: usize, a: f64) -> (Color, usize, usize, Vec<Action>) {
        // i番目のウェルを使用. k2の絵の具を追加 => aに分割 => k1の絵の具を追加 => 提出 => 仕切りを戻す
        let mut cnt = 0;
        let mut turn = 0;
        let mut actions: Vec<Action> = Vec::new();
        let well_size= self.well_size;

        // 係数はwell_size倍する
        let j = (a*well_size as f64).round() as usize;
        assert!(j >= 2, "two paint2 not j < 2, j: {}, a: {}", j, a);
        let j = self.start_j + j;

        // k2の絵の具
        if k2 != usize::MAX {  // ウェルに色が残っていない時に追加
            actions.push(Action::add_color(i, self.start_j, k2));
            turn += 1;
            cnt += 1;
        }
        if j < 19 {
            actions.push(Action::toggle_separator(i, j-1, i, j));  // 必要な割合に分ける
            turn += 1;
        }

        // k1の絵の具追加、提出
        actions.push(Action::add_color(i, self.start_j, k1));
        actions.push(Action::give_paint(i, self.start_j));
        turn += 2;
        cnt += 1;

        // 仕切り戻す
        if j < 19 {
            actions.push(Action::toggle_separator(i, j-1, i, j));  // 必要な割合に分ける
            turn += 1;
        }

        // 作成した色を再現
        let mut color = self.base_color[k1];
        let color2 = if k2 == usize::MAX {
            self.remain_color[i]
        } else {
            self.base_color[k2]
        };
        color.add_color(color2, j as f64 / self.well_size as f64);
        let all_gram = 1.0 + j as f64 / self.well_size as f64;
        color.c /= all_gram;
        color.m /= all_gram;
        color.y /= all_gram;
        let make_color = color;
        
        let r1 = j as f64 / self.well_size as f64;
        let r2 = 1.0 - r1;
        let remain_color = Color { c: color.c*r1 + color2.c*r2, m: color.m*r1 + color2.m*r2, y: color.y*r1 + color2.y*r2 };
        if a != f64::MAX {
            let rate = (self.well_size-(j-self.start_j)) as f64 / (j-self.start_j) as f64;
            color.add_color(color2, rate);
            let all_gram = 1.0+rate;
            color.c /= all_gram;
            color.m /= all_gram;
            color.y /= all_gram;
        }
        self.remain_color[i] = remain_color;
        self.remain_gram[i] = 1.0;
        if self.remain_gram[i] < 0.0 {
            self.remain_gram[i] = 0.0;
        }

        (make_color, cnt, turn, actions)
    }

    fn give_paint(&mut self, i: usize) -> (Color, usize, usize, Vec<Action>) {
        let cnt = 0;
        let turn = 1;
        let color = self.remain_color[i];
        assert!(self.remain_gram[i]==1.0);
        let actions: Vec<Action> = vec![Action::give_paint(i, self.start_j)];
        self.remain_color[i] = Color { c: 0.0, m: 0.0, y: 0.0 };
        self.remain_gram[i] = 0.0;

        (color, cnt, turn, actions)
    }
}

#[derive(Clone)]
struct Palette7 {
    base_color: Vec<Color>,
    base_gram: Vec<f64>,
    well_size: usize,
    well_cnt: usize,
}

impl Palette7 {
    fn new(own: &Vec<Color>, well_size: usize, well_cnt: usize) -> Self {
        let k = own.len();
        let mut base_color: Vec<Color> = Vec::new();
        for &o in own.iter() {
            base_color.push(o);
        }
        let base_gram: Vec<f64> = vec![0.0; k];

        Self { base_color, base_gram, well_size, well_cnt }
    }

    fn entrance(&self, k: usize) -> (usize, usize, usize, usize) {
        let i = k * self.well_cnt;
        let j = self.well_size-1;

        (i, j, i, j+1)
    }

    fn partition(&self, k: usize, r: usize) -> (usize, usize, usize, usize) {
        let (mut i1, _, _, _) = self.entrance(k);
        let di = (r-1) / self.well_size;
        i1 += di;

        let mut j1 = (r-1) % self.well_size + 1;
        if di%2 == 0 { j1 = self.well_size - j1 - 1; }

        let (i2, j2) = if j1%self.well_size == 0 {
            if di == self.well_cnt-1 {
                (i1, j1)
            } else {
                (i1+1, j1)
            }
        } else if di%2 == 0 {
                (i1, j1-1)
        } else {
            (i1, j1+1)
        };

        (i1, j1, i2, j2)
    }
    
    fn init(&self, n: usize) -> Palette {
        let mut v = vec![vec![false; n-1]; n];
        for i in 0..n {
            v[i][self.well_size-1] = true;
            if self.well_size < 19 {
                v[i][self.well_size] = true;
            }
        }
        let mut h = vec![vec![false; n]; n-1];
        for i in 0..(n-1) {
            for j in 0..self.well_size {
                let di = i%self.well_cnt;
                if (i+1)%self.well_cnt!=0 && ((j==0 && di%2==0) || (j==self.well_size-1 && di%2==1)) {
                    continue;
                }
                h[i][j] = true;
            }
        }

        Palette { v, h }
    }

    fn make_paint(&mut self, t: &Color, k1: usize, k2: usize, k3: usize, k4: usize, a: f64, b: f64, c: f64, d: f64) -> (Color, usize, usize, Vec<Action>) {
        let mut cnt = 0;
        let mut turn = 0;
        let mut actions: Vec<Action> = Vec::new();
        let well_size= self.well_size*self.well_cnt;

        // 係数はwell_size倍する
        let a = [(a*well_size as f64), (b*well_size as f64), (c*well_size as f64), (d*well_size as f64)];
        let mut partition: Vec<(usize, usize)> = Vec::new();
        let mut use_grams: Vec<f64> = Vec::new();
        let mut all_gram = 0.0;

        // 切り上げ
        let mut opt_j = vec![0, 0, 0, 0];
        let mut opt_e = f64::MAX;
        let mut js: Vec<usize> = Vec::new();
        let mut use_grams: Vec<f64> = Vec::new();
        let mut all_gram = 0.0;
        for (i, &k) in [k1, k2, k3, k4].iter().enumerate() {
            let mut g = self.base_gram[k];
            if g < a[i]/well_size as f64 {
                g += 1.0;
            }
            let j = (a[i]/g).ceil() as usize;
            js.push(j);
            let u_g = g * j as f64 / well_size as f64;
            use_grams.push(u_g);
            all_gram += u_g;
        }
        eprintln!("js: {:?}", js);
        // 色を再現
        let mut color = Color::new(0.0, 0.0, 0.0);
        for (i, &k) in [k1, k2, k3, k4].iter().enumerate() {
            eprintln!("k: {}, use_grams: {}", k, use_grams[i]);
            color.add_color(self.base_color[k], use_grams[i]);
        }
        eprintln!("color: {:?}", color);
        color.c /= all_gram;
        color.m /= all_gram;
        color.y /= all_gram;
        
        let e = color.dist(t);
        if opt_e > e {
            opt_e = e;
            opt_j = js;
        }

        for (i, &k) in [k1, k2, k3, k4].iter().enumerate() {
            // 絵の具の量が足りない場合は追加
            if self.base_gram[k] < a[i]/well_size as f64 {
                let (i1, j1, _, _) = self.entrance(k);
                actions.push(Action::add_color(i1, j1, k));
                cnt += 1;
                turn += 1;
                self.base_gram[k] += 1.0;
            }
            // 必要な絵の具の量にするため、仕切りを追加
            // let mut j = (a[i]/self.base_gram[k]).floor() as usize;
            let mut j = opt_j[i];
            all_gram += self.base_gram[k] * j as f64 / well_size as f64;
            if all_gram < 1.0 { j += 1; }
            if j < well_size && j > 0 {
                let (i1, j1, i2, j2) = self.partition(k, j);
                // eprintln!("paration: {:?}", (i1, j1, i2, j2));
                actions.push(Action::toggle_separator(i1, j1, i2, j2));  // 必要な割合に分ける
                partition.push((k, j));
                turn += 1;
            }
            let (i1, j1, i2, j2) = self.entrance(k);
            // eprintln!("entrance: {:?}", (i1, j1, i2, j2));
            actions.push(Action::toggle_separator(i1, j1, i2, j2));  // 混ぜる場所に繋げる
            turn += 1;
            let use_gram = self.base_gram[k] * j as f64 / well_size as f64;
            self.base_gram[k] -= use_gram;  // 残る絵の具の量を計算
            use_grams.push(use_gram);
        }

        // 絵の具の提出、残りの破棄
        actions.push(Action::give_paint(0, self.well_size));
        actions.push(Action::discard_paint(0, self.well_size));
        turn += 2;

        // 仕切りを元に戻す
        for (i, &k) in [k1, k2, k3, k4].iter().enumerate() {
            let (i1, j1, i2, j2) = self.entrance(k);
            actions.push(Action::toggle_separator(i1, j1, i2, j2));  // 混ぜる場所から隔離
            turn += 1;
        }
        for &(k, j) in partition.iter() {
            let (i1, j1, i2, j2) = self.partition(k, j);
            actions.push(Action::toggle_separator(i1, j1, i2, j2));  // 各絵の具の場所を元に戻す
            turn += 1;
        }

        // 作成した色を再現
        let mut color = Color::new(0.0, 0.0, 0.0);
        for (i, &k) in [k1, k2, k3, k4].iter().enumerate() {
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

    fn two_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        let primes = vec![1, 2, 3, 5];
        let palette = Palette2::new(&self.own, primes);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for (i, &target_color) in self.target.iter().enumerate() {
            let mut opt_eval= usize::MAX;
            let mut opt_action: Vec<Action> = Vec::new();
            let mut opt_score = score;
            let mut opt_turn = 0;
            let mut opt_color = Color::new(0.0, 0.0, 0.0);
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
                                opt_color = color;
                            }
                        }
                    }
                }
            }
            actions.extend(opt_action);
            score = opt_score;
            turn += opt_turn;
            // if i < 10 {
            //     eprintln!("two paint: i: {}, e: {}", i, opt_color.dist(&self.target[i]));
            // }
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
            // if i < 10 {
            //     eprintln!("base paint2: i: {}, e: {}", i, color.dist(&self.target[i]));
            // }
        }
        
        (palette.init(self.n), actions, score, turn)
    }

    fn two_paint2(&self) -> (Palette, Vec<Action>, Score, usize, Vec<(usize, usize, usize, f64)>) {
        // 関数定義
        fn dot(v1: Color, v2: Color) -> f64 {
            v1.c*v2.c + v1.m*v2.m + v1.y*v2.y
        }

        fn subtract(v1: Color, v2: Color) -> Color {
            Color { c: v1.c-v2.c, m: v1.m-v2.m, y: v1.y-v2.y }
        }

        fn mixing_2color(v1: Color, v2: Color, a: f64) -> Color {
            let mut color = Color { c: 0.0, m: 0.0, y: 0.0 };
            color.add_color(v1, 1.0-a);
            color.add_color(v2, a);

            color
        }

        fn opt_2color(t: Color, v1: Color, v2: Color) -> (Color, f64) {
            let t_v1 = subtract(t, v1);
            let v2_v1 = subtract(v2, v1);

            let a = dot(t_v1, v2_v1) / dot(v2_v1, v2_v1);

            let color = mixing_2color(v1, v2, a);
            (color, a)
        }

        // 各目標の色の2つの絵の具とその割合を算出
        let mut palette = Palette6::new(&self.own, 0, 20);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        let mut two_paint_params: Vec<(usize, usize, usize, f64)> = Vec::new();
        let mut estimate_e = 0.0;

        for (i, &t) in self.target.iter().enumerate() {
            let mut opt_e = f64::MAX;
            let mut opt_params = (0, 0, 0, f64::MAX);

            // パレットのウェルに色がある場合を順に処理
            let mut empty_well = false;
            for j in 0..self.n {
                let remain_gram = palette.remain_gram[j];
                if remain_gram > 0.0 {
                    // そのまま使える場合
                    let remain_color = palette.remain_color[j];
                    let e = t.dist(&remain_color);
                    if opt_e > e {
                        opt_e = e;
                        opt_params = (j, usize::MAX, usize::MAX, f64::MAX);
                    }

                    // 残っている色と混ぜる
                    for k1 in 0..self.k {
                        let (color, a) = opt_2color(t, self.own[k1], remain_color);
                        if !(0.0..0.5).contains(&a) { continue; }
                        if (((a/(1.0-a)) * palette.well_size as f64).round() as usize ) < 2 { continue; }
                        let e = t.dist(&color);
                        if opt_e > e {
                            opt_e = e;
                            opt_params = (j, k1, usize::MAX, a/(1.0-a));  // パレット用の係数に変換
                        }
                    }
                } else {  // ウェルが空の場合
                    if empty_well { continue; }
                    for k1 in 0..self.k {
                        for k2 in 0..self.k {
                            if k1 == k2 { continue; }
                            let v1 = self.own[k1];
                            let v2 = self.own[k2];

                            let (color, a) = opt_2color(t, v1, v2);
                            if !(0.0..0.5).contains(&a) { continue; }
                            if (((a/(1.0-a)) * palette.well_size as f64).round() as usize) < 2 { continue; }
                            let e = t.dist(&color);
                            if opt_e > e {
                                opt_e = e;
                                opt_params = (j, k1, k2, a/(1.0-a));  // パレット用の係数に変換
                            }
                        }
                    }
                    empty_well = true;  // 空のウェルの処理は一度だけ実施
                }
            }
            two_paint_params.push(opt_params);
            let (opt_i, opt_k1, opt_k2, opt_a) = opt_params;
            // eprintln!("opt_a: {}, {}", opt_a, opt_a*palette.well_size as f64);
            let (color, cnt, tt, action) = if opt_k1 == usize::MAX {
                palette.give_paint(opt_i)
            } else {
                palette.make_paint(opt_i, opt_k1, opt_k2, opt_a)
            };
            score.add_score(self.target[i], color, cnt);
            turn += tt;
            actions.extend(action);
            estimate_e += opt_e;
            // if i < 10 {
            //     eprintln!("params: {:?}", opt_params);
            //     eprintln!("two paint2: i: {}, e: {}, estimate_e: {}, color: {:?}", i, color.dist(&self.target[i]), opt_e, (color.c, color.m, color.y));
            // }
        }
        eprintln!("two paint2 estimate_e: {}", estimate_e);

        (palette.init(self.n), actions, score, turn, two_paint_params)
    }

    fn three_paint(&self) -> (Palette, Vec<Action>, Score, usize, Vec<(usize, usize, usize, f64, f64)>) {
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
        }
        eprintln!("estimate_e: {}", estimate_e);

        // Actionsの構築
        let mut palette = Palette5::new(&self.own, 15);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        for i in 0..self.h { 
            let (k1, k2, k3, a, b) = three_paint_params[i];
            let (color, cnt, t, action) = palette.make_paint(k1, k2, k3, a, b);
            score.add_score(self.target[i], color, cnt);
            turn += t;
            actions.extend(action);
            // if i < 10 {
            //     eprintln!("three paint: i: {}, e: {}", i, color.dist(&self.target[i]));
            // }
        }

        (palette.init(self.n), actions, score, turn, three_paint_params)
    }

    fn four_paint(&self) -> (Palette, Vec<Action>, Score, usize) {
        // 関数定義
        fn dot(v1: Color, v2: Color) -> f64 {
            v1.c*v2.c + v1.m*v2.m + v1.y*v2.y
        }

        fn subtract(v1: Color, v2: Color) -> Vec<f64> {
            vec![v1.c-v2.c, v1.m-v2.m, v1.y-v2.y]
        }

        fn permutation_sign(perm: &[usize]) -> f64 {
            let mut inversions = 0;
            for i in 0..perm.len() {
                for j in (i + 1)..perm.len() {
                    if perm[i] > perm[j] {
                        inversions += 1;
                    }
                }
            }
            if inversions % 2 == 0 { 1.0 } else { -1.0 }
        }

        fn determinate(a: &Vec<Vec<f64>>) -> f64 {
            let n = a.len();
            let mut det = 0.0;
            for perm in (0..n).permutations(n) {
                let sgn = permutation_sign(&perm);
                let mut tmp = sgn;
                for i in 0..n {
                    tmp *= a[i][perm[i]];
                }
                det += tmp;
            }

            det
        }

        fn inverse_3matrix(a: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
            let a = Matrix3::new(
                a[0][0], a[0][1], a[0][2],
                a[1][0], a[1][1], a[1][2],
                a[2][0], a[2][1], a[2][2],
            );
            let b = a.try_inverse().unwrap();
            let b = (0..3)
                .map(|i| (0..3).map(|j| b[(i, j)]).collect()).collect();;

            // let det = determinate(a);
            // let mut b = vec![
            //     vec![a[0][0]*a[2][2]-a[1][2]*a[2][1], a[0][2]*a[2][1]-a[0][1]*a[2][2], a[0][1]*a[1][2]-a[0][2]*a[1][1]],
            //     vec![a[1][2]*a[2][0]-a[1][0]*a[2][2], a[0][0]*a[2][2]-a[0][2]*a[2][0], a[0][2]*a[1][0]-a[0][0]*a[1][2]],
            //     vec![a[1][0]*a[2][1]-a[1][1]*a[2][0], a[0][1]*a[2][0]-a[0][0]*a[2][1], a[0][0]*a[1][1]-a[0][1]*a[1][0]],
            // ];
            // eprintln!("b: {:?}", b);
            // for i in 0..b.len() {
            //     for j in 0..b[0].len() {
            //         b[i][j] /= det;
            //     }
            // }

            b
        }

        fn matrix_multi(a1: &Vec<Vec<f64>>, a2: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
            let mut m: Vec<Vec<f64>> = Vec::new();
            for i in 0..a1.len() {
                let mut vec: Vec<f64> = Vec::new();
                for j in 0..a2[0].len() {
                    let mut tmp = 0.0;
                    for k in 0..a1[0].len() {
                        tmp += a1[i][k]*a2[k][j];
                    }
                    vec.push(tmp);
                }
                m.push(vec);
            }

            m
        }

        fn mixing_4color(v1: Color, v2: Color, v3: Color, v4: Color, a: f64, b: f64, c: f64, d: f64) -> Color {
            let mut color = Color { c: 0.0, m: 0.0, y: 0.0 };
            color.add_color(v1, a);
            color.add_color(v2, b);
            color.add_color(v3, c);
            color.add_color(v4, d);

            color
        }

        fn opt_4color(t: Color, v1: Color, v2: Color, v3: Color, v4: Color) -> (Color, f64, f64, f64, f64) {
            let t_v4 = subtract(t, v4);
            let v1_v4 = subtract(v1, v4);
            let v2_v4 = subtract(v2, v4);
            let v3_v4 = subtract(v3, v4);
            
            let mut m: Vec<Vec<f64>> = Vec::new();
            for i in 0..3 {
                m.push(vec![v1_v4[i], v2_v4[i], v3_v4[i]]);
            }
            let inv_m = inverse_3matrix(&m);
            let e = matrix_multi(&m, &inv_m);

            let mut tmp_t_v4 = Vec::new();
            for &ti in t_v4.iter() {
                tmp_t_v4.push(vec![ti]);
            }
            let t_v4 = tmp_t_v4;

            let a = matrix_multi(&inv_m, &t_v4);
            let (a, b, c) = (a[0][0], a[1][0], a[2][0]);
            let d = 1.0-a-b-c;

            let color = mixing_4color(v1, v2, v3, v4, a, b, c, d);
            (color, a, b, c, d)
        }

        // 各目標の色の4つの絵の具とその割合を算出
        let mut estimate_e = 0.0;
        let mut four_paint_params: Vec<(usize, usize, usize, usize, f64, f64, f64, f64)> = Vec::new();

        for (i, &t) in self.target.iter().enumerate() {
            let mut flg = false;
            let mut param = (0, 1, 2, 3, 1.0, 0.0, 0.0, 0.0);

            for k1 in 0..self.k {
                for k2 in (k1+1)..self.k {
                    for k3 in (k2+1)..self.k {
                        for k4 in (k3+1)..self.k {
                            let v1 = self.own[k1];
                            let v2 = self.own[k2];
                            let v3 = self.own[k3];
                            let v4 = self.own[k4];

                            let (color, a, b, c, d) = opt_4color(t, v1, v2, v3, v4);
                            if !(0.0..1.0).contains(&a) || !(0.0..1.0).contains(&b) || !(0.0..1.0).contains(&c) || !(0.0..1.0).contains(&d) {
                                param = (k1, k2, k3, k4,
                                    if a < 0.0 { 0.0 } else { a },
                                    if b < 0.0 { 0.0 } else { b },
                                    if c < 0.0 { 0.0 } else { c },
                                    if d < 0.0 { 0.0 } else { d },
                                );
                                continue;
                            }
                            let e = t.dist(&color);
                            flg = true;
                            estimate_e += e;
                            param = (k1, k2, k3, k4, a, b, c, d);
                            break;
                        }
                        if flg { break; }
                    }
                    if flg { break; }
                }
                if flg { break; }
            }
            four_paint_params.push(param);
        }
        eprintln!("estimate_e: {}", estimate_e);

        // Actionsの構築
        let well_size = self.n-1;
        let well_cnt = self.n / self.k;
        eprintln!("well_size: {}, {}, {}", well_size, well_cnt, well_size*well_cnt);
        let well_size = 19;
        let mut palette = Palette7::new(&self.own, well_size, well_cnt);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        let mut real_e = 0.0;

        for i in 0..self.h { 
            let (k1, k2, k3, k4, a, b, c, d) = four_paint_params[i];
            let (color, cnt, t, action) = palette.make_paint(&self.target[i], k1, k2, k3, k4, a, b, c, d);
            score.add_score(self.target[i], color, cnt);
            turn += t;
            actions.extend(action);
            real_e += color.dist(&self.target[i]);
            if i == 0 {
                eprintln!("i: {}, e: {}", i, color.dist(&self.target[i]));
                break;
            }
        }
        eprintln!("real_e: {}", real_e);

        (palette.init(self.n), actions, score, turn)
    }

    fn opt_paint(&self, three_paint_params: &Vec<(usize, usize, usize, f64, f64)>) -> (Palette, Vec<Action>, Score, usize) {
        // 関数定義
        fn dot(v1: Color, v2: Color) -> f64 {
            v1.c*v2.c + v1.m*v2.m + v1.y*v2.y
        }

        fn subtract(v1: Color, v2: Color) -> Color {
            Color { c: v1.c-v2.c, m: v1.m-v2.m, y: v1.y-v2.y }
        }

        fn mixing_2color(v1: Color, v2: Color, a: f64) -> Color {
            let mut color = Color { c: 0.0, m: 0.0, y: 0.0 };
            color.add_color(v1, 1.0-a);
            color.add_color(v2, a);

            color
        }

        fn opt_2color(t: Color, v1: Color, v2: Color) -> (Color, f64) {
            let t_v1 = subtract(t, v1);
            let v2_v1 = subtract(v2, v1);

            let a = dot(t_v1, v2_v1) / dot(v2_v1, v2_v1);

            let color = mixing_2color(v1, v2, a);
            (color, a)
        }

        // palette5(three_paint), palette6(two_paint)のうち一番良い結果を採用
        // ターン数の重みや調整も入れる
        let mut palette5 = Palette5::new(&self.own, 10);
        let mut palette6 = Palette6::new(&self.own, 10, 9);
        let mut score = Score::new(self.h, self.d);
        let mut actions: Vec<Action> = Vec::new();
        let mut turn = 0;
        let mut backup_palette5 = palette5.clone();
        let mut backup_palette6 = palette6.clone();
        for i in 0..self.h { 
            // palette6
            let mut opt_e = f64::MAX;
            let mut opt_params = (0, 0, 0, f64::MAX);

            // パレットのウェルに色がある場合を順に処理
            let mut empty_well = false;
            for j in 0..self.n {
                let remain_gram = palette6.remain_gram[j];
                if remain_gram > 0.0 {
                    // そのまま使える場合
                    let remain_color = palette6.remain_color[j];
                    let e = self.target[i].dist(&remain_color);
                    if opt_e > e {
                        opt_e = e;
                        opt_params = (j, usize::MAX, usize::MAX, f64::MAX);
                    }

                    // 残っている色と混ぜる
                    for k1 in 0..self.k {
                        let (color, a) = opt_2color(self.target[i], self.own[k1], remain_color);
                        if !(0.0..0.5).contains(&a) { continue; }
                        if (((a/(1.0-a)) * palette6.well_size as f64).round() as usize ) < 2 { continue; }
                        let e = self.target[i].dist(&color);
                        if opt_e > e {
                            opt_e = e;
                            opt_params = (j, k1, usize::MAX, a/(1.0-a));  // パレット用の係数に変換
                        }
                    }
                } else {  // ウェルが空の場合
                    if empty_well { continue; }
                    for k1 in 0..self.k {
                        for k2 in 0..self.k {
                            if k1 == k2 { continue; }
                            let v1 = self.own[k1];
                            let v2 = self.own[k2];

                            let (color, a) = opt_2color(self.target[i], v1, v2);
                            if !(0.0..0.5).contains(&a) { continue; }
                            if (((a/(1.0-a)) * palette6.well_size as f64).round() as usize) < 2 { continue; }
                            let e = self.target[i].dist(&color);
                            if opt_e > e {
                                opt_e = e;
                                opt_params = (j, k1, k2, a/(1.0-a));  // パレット用の係数に変換
                            }
                        }
                    }
                    empty_well = true;  // 空のウェルの処理は一度だけ実施
                }
            }
            let (opt_i, opt_k1, opt_k2, opt_a) = opt_params;
            // eprintln!("opt_a: {}, {}", opt_a, opt_a*palette.well_size as f64);
            let (p2_color, p2_cnt, p2_t, p2_action) = if opt_k1 == usize::MAX {
                palette6.give_paint(opt_i)
            } else {
                palette6.make_paint(opt_i, opt_k1, opt_k2, opt_a)
            };
            let p2_e = opt_e;
            
            // palette5
            let (k1, k2, k3, a, b) = three_paint_params[i];
            let (p3_color, p3_cnt, p3_t, p3_action) = palette5.make_paint(k1, k2, k3, a, b);
            let p3_e = self.target[i].dist(&p3_color);

            if p2_e > p3_e && self.turn > turn+p3_t+(self.h-(i+1))*4 {
                score.add_score(self.target[i], p3_color, p3_cnt);
                turn += p3_t;
                actions.extend(p3_action);
                palette6 = backup_palette6.clone();
                backup_palette5 = palette5.clone();
            } else {
                score.add_score(self.target[i], p2_color, p2_cnt);
                turn += p2_t;
                actions.extend(p2_action);
                palette5 = backup_palette5.clone();
                backup_palette6 = palette6.clone();
            }
        }

        (palette5.init(self.n), actions, score, turn)
    }

    fn solve(&mut self) {
        eprintln!("start solve: {:.2?} s", self.timer.elapsed());
        (self.palette, self.actions, self.score, self.turn) = self.base_paint2(); 
        eprintln!("{:.2?} s: base paint2: score: {}, turn: {}", self.timer.elapsed(), self.score, self.turn);

        let (palette, actions, score, turn) = self.two_paint();
        eprintln!("{:.2?} s: two paint: {}, turn: {}", self.timer.elapsed(), score, turn);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn, three_paint_params) = self.three_paint();
        eprintln!("{:.2?} s: three paint: {}, turn: {}", self.timer.elapsed(), score, turn);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn) = self.opt_paint(&three_paint_params);
        eprintln!("{:.2?} s: opt paint: {}, turn: {}", self.timer.elapsed(), score, turn);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn, two_paint_params) = self.two_paint2();
        eprintln!("{:.2?} s: two paint2: {}, turn: {}", self.timer.elapsed(), score, turn);
        if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        }

        let (palette, actions, score, turn) = self.four_paint();
        eprintln!("{:.2?} s: four paint: {}, turn: {}", self.timer.elapsed(), score, turn);
        // if score < self.score && turn <= self.t {
            self.palette = palette;
            self.actions = actions;
            self.score = score;
            self.turn = turn;
        // }
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
