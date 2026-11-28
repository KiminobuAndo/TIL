//! `notes/rust-traits-generics.md` のサンプルを実際に動かすための crate。
//!
//! 実行: `cargo run -p traits`
//! テスト: `cargo test -p traits`
//!
//! `// error[...]` と書いてある行はコンパイルエラーになる例。
//! コメントを外して `cargo build -p traits` すると、そのエラー番号が出ることを確かめられる。

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign};

fn main() {
    println!("== 1. trait の定義と実装 ==");
    t1_basic();
    t1_assoc();

    println!("\n== 2. ジェネリクス ==");
    g2_functions();
    g2_structs();

    println!("\n== 3. trait 境界 ==");
    b3_bounds();
    b3_impl_trait();
    b3_conditional_impl();
    b3_blanket();

    println!("\n== 4. 静的ディスパッチと動的ディスパッチ ==");
    d4_static_vs_dynamic();

    println!("\n== 5. 関連型と標準 trait ==");
    a5_iterator();
    a5_derive();
    a5_from_into();
    a5_operator();

    println!("\n== 6. orphan rule と newtype ==");
    o6_newtype();

    println!("\n== 8. 設計の指針: 金額型と試算表 ==");
    d8_trial_balance();
}

// ---------------------------------------------------------------------------
// 1. trait の定義と実装
// ---------------------------------------------------------------------------

/// 「面積を持つ」という振る舞いの約束
trait Shape {
    // 実装側が必ず書くメソッド（本体なし）
    fn area(&self) -> f64;

    // デフォルト実装つきのメソッド。上書きしてもしなくてもよい
    fn name(&self) -> String {
        "図形".to_string()
    }

    // デフォルト実装から、必須メソッドを呼べる
    fn describe(&self) -> String {
        format!("{}（面積 {:.2}）", self.name(), self.area())
    }
}

struct Circle {
    r: f64,
}

struct Rect {
    w: f64,
    h: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.r * self.r
    }
    fn name(&self) -> String {
        "円".to_string()
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    // name は書かない → デフォルトの "図形" が使われる
}

// struct Triangle;
// impl Shape for Triangle {} // error[E0046]: not all trait items implemented, missing: `area`

fn t1_basic() {
    let c = Circle { r: 1.0 };
    let r = Rect { w: 2.0, h: 3.0 };
    println!("{}", c.describe());
    println!("{}", r.describe());

    // メソッド構文 r.area() は Shape::area(&r) の糖衣構文
    println!("Shape::area(&r) = {}", Shape::area(&r));
}

/// 関連定数と関連関数（self を取らない）を持つ trait
trait Unit {
    const SYMBOL: &'static str; // 関連定数: 実装ごとに値を決める
    fn from_raw(v: i64) -> Self; // 関連関数: Self は「この trait を実装する型」
    fn raw(&self) -> i64;

    fn show(&self) -> String {
        format!("{}{}", self.raw(), Self::SYMBOL)
    }
}

struct Yen(i64);
struct Meter(i64);

impl Unit for Yen {
    const SYMBOL: &'static str = "円";
    fn from_raw(v: i64) -> Self {
        Yen(v)
    }
    fn raw(&self) -> i64 {
        self.0
    }
}

impl Unit for Meter {
    const SYMBOL: &'static str = "m";
    fn from_raw(v: i64) -> Self {
        Meter(v)
    }
    fn raw(&self) -> i64 {
        self.0
    }
}

/// ジェネリクスの中で「型 U から値を作る」ことができる（JS の interface ではできない）
fn doubled<U: Unit>(u: &U) -> U {
    U::from_raw(u.raw() * 2)
}

fn t1_assoc() {
    let y = Yen::from_raw(1500);
    let m = <Meter as Unit>::from_raw(42); // 完全修飾構文（どの trait の関数か明示）
    println!("{} / {}", y.show(), m.show());
    println!("Yen::SYMBOL = {:?}", Yen::SYMBOL);
    println!("doubled = {} / {}", doubled(&y).show(), doubled(&m).show());
}

// ---------------------------------------------------------------------------
// 2. ジェネリクス
// ---------------------------------------------------------------------------

/// 最大値。空なら None（overview 版の list[0] は空で panic するので改良）
fn largest<T: PartialOrd + Copy>(list: &[T]) -> Option<T> {
    let mut iter = list.iter().copied();
    let mut max = iter.next()?;
    for x in iter {
        if x > max {
            max = x;
        }
    }
    Some(max)
}

// 境界が無いと、T 同士を比較できるか分からない
// fn bad_max<T>(a: T, b: T) -> T {
//     if a > b { a } else { b } // error[E0369]: binary operation `>` cannot be applied to type `T`
// }

fn g2_functions() {
    println!("largest(&[3, 7, 2])       = {:?}", largest(&[3, 7, 2]));
    println!("largest(&[1.5, -0.5])     = {:?}", largest(&[1.5, -0.5]));
    println!("largest(&['a', 'z', 'm']) = {:?}", largest(&['a', 'z', 'm']));
    // 引数から T が推論できないときはターボフィッシュ ::<> で指定する
    println!("largest::<i32>(&[])       = {:?}", largest::<i32>(&[]));

    // String は Copy ではないので呼べない
    // largest(&["a".to_string()]); // error[E0277]: the trait bound `String: Copy` is not satisfied

    // 戻り値の型で T が決まる例: parse::<T> / collect::<T>
    let n = "42".parse::<u8>();
    let v: Vec<i32> = "1 2 3".split(' ').map(|s| s.parse().unwrap()).collect();
    println!("parse::<u8> = {n:?}, collect = {v:?}");
}

/// 型引数を持つ struct
#[derive(Debug)]
struct Pair<T> {
    a: T,
    b: T,
}

// impl<T> … どんな T の Pair にも使えるメソッド
impl<T> Pair<T> {
    fn new(a: T, b: T) -> Self {
        Self { a, b }
    }
    fn swap(self) -> Self {
        Self {
            a: self.b,
            b: self.a,
        }
    }
}

// impl Pair<f64> … 具体的な型のときだけ生えるメソッド
impl Pair<f64> {
    fn mean(&self) -> f64 {
        (self.a + self.b) / 2.0
    }
}

fn g2_structs() {
    let p = Pair::new(1, 2); // Pair<i32>
    let q = Pair::new("left", "right"); // Pair<&str>
    let f = Pair::new(1.0, 4.0); // Pair<f64>
    println!("{:?} → {:?}", Pair::new(1, 2), p.swap());
    println!("{q:?}");
    println!("Pair<f64>.mean() = {}", f.mean());
    // Pair::new(1, 2).mean(); // error[E0599]: no method named `mean` found for struct `Pair<{integer}>`
    // Pair::new(1, "x");      // error[E0308]: mismatched types（a と b は同じ T）

    // Option<T> / Result<T, E> / Vec<T> も同じ仕組みのジェネリックな型
    let xs: Vec<Option<char>> = [Some('a'), None].to_vec();
    println!("Vec<Option<char>> = {xs:?}");
}

// ---------------------------------------------------------------------------
// 3. trait 境界
// ---------------------------------------------------------------------------

/// 境界をインラインで書く
fn join_all<T: fmt::Display>(items: &[T], sep: &str) -> String {
    items
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

/// 境界が長くなったら where 句
fn count_unique<T>(items: &[T]) -> usize
where
    T: Eq + std::hash::Hash,
{
    items.iter().collect::<HashSet<_>>().len()
}

fn b3_bounds() {
    println!("join_all = {}", join_all(&[1, 2, 3], " + "));
    println!("join_all = {}", join_all(&["借方", "貸方"], " / "));
    println!("count_unique = {}", count_unique(&["a", "b", "a", "c", "b"]));
    // f64 は Eq / Hash を実装していない（NaN != NaN のため）
    // count_unique(&[1.0, 2.0]); // error[E0277]: the trait bound `{float}: Eq` is not satisfied
}

/// impl Trait（引数位置）: 「IntoIterator を実装した何か」。ジェネリクスの短縮記法
fn total_len(words: impl IntoIterator<Item = impl AsRef<str>>) -> usize {
    words.into_iter().map(|w| w.as_ref().chars().count()).sum()
}

/// impl Trait（戻り値位置）: 具体型を隠して「Iterator である」とだけ約束する
fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
}

/// クロージャは型に名前が無いので、返すには impl Fn が必要
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// 戻り値の impl Trait は「1 つの具体型」でなければならない
// fn pick(flag: bool) -> impl Iterator<Item = u32> {
//     if flag { (0..3).map(|x| x) } else { vec![1].into_iter() }
//     // error[E0308]: `if` and `else` have incompatible types
// }

fn b3_impl_trait() {
    // Vec<&str> も配列 [String; 2] も渡せる
    println!("total_len = {}", total_len(vec!["仕訳", "試算表"]));
    println!(
        "total_len = {}",
        total_len(["abc".to_string(), "de".to_string()])
    );
    println!("evens(10) = {:?}", evens(10).collect::<Vec<_>>());
    let add5 = make_adder(5);
    println!("add5(37) = {}", add5(37));
}

// 条件付きメソッド: T が Display + PartialOrd のときだけ使える
impl<T: fmt::Display + PartialOrd> Pair<T> {
    fn show_larger(&self) -> String {
        if self.a >= self.b {
            format!("a = {}", self.a)
        } else {
            format!("b = {}", self.b)
        }
    }
}

fn b3_conditional_impl() {
    println!("{}", Pair::new(3, 8).show_larger());
    println!("{}", Pair::new("z", "a").show_larger());
    // Circle は Display も PartialOrd も無いので show_larger は生えない
    // Pair::new(Circle { r: 1.0 }, Circle { r: 2.0 }).show_larger(); // error[E0599]
}

/// ブランケット実装: 「Debug を実装する全ての型」に Label を一括で実装する
trait Label {
    fn label(&self) -> String;
}

impl<T: fmt::Debug> Label for T {
    fn label(&self) -> String {
        format!("<<{self:?}>>")
    }
}

fn b3_blanket() {
    // 自分で impl していない i32 や Vec にも label が生える
    println!("{} {} {}", 42.label(), "hi".label(), [1, 2].label());
    // 標準ライブラリの例: impl<T: Display + ?Sized> ToString for T
    // → Display を実装すれば to_string() が自動で使える
}

// ---------------------------------------------------------------------------
// 4. 静的ディスパッチと動的ディスパッチ
// ---------------------------------------------------------------------------

/// 静的ディスパッチ: S ごとに別の関数がコンパイル時に生成される（単相化）
fn total_area_static<S: Shape>(shapes: &[S]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

/// 動的ディスパッチ: 実行時に vtable を引いて area を呼ぶ
fn total_area_dyn(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

/// &dyn Shape でも受けられる（Box に入れなくてよい）
fn print_shape(s: &dyn Shape) {
    println!("  - {}", s.describe());
}

fn d4_static_vs_dynamic() {
    // 同じ型だけなら静的ディスパッチで足りる
    let rects = [Rect { w: 1.0, h: 2.0 }, Rect { w: 3.0, h: 4.0 }];
    println!("total_area_static = {}", total_area_static(&rects));

    // 異なる型を 1 つのコレクションに入れるには dyn が必要
    // let mixed = [Circle { r: 1.0 }, Rect { w: 1.0, h: 1.0 }]; // error[E0308]: mismatched types
    let mixed: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { r: 1.0 }),
        Box::new(Rect { w: 2.0, h: 3.0 }),
    ];
    println!("total_area_dyn = {:.2}", total_area_dyn(&mixed));
    for s in &mixed {
        print_shape(s.as_ref());
    }

    // &dyn Trait は「データへのポインタ + vtable へのポインタ」の fat pointer
    println!(
        "size_of::<&Rect>() = {}, size_of::<&dyn Shape>() = {}",
        std::mem::size_of::<&Rect>(),
        std::mem::size_of::<&dyn Shape>()
    );

    // Self を返す関連関数や関連定数を持つ trait は dyn にできない
    // let u: Box<dyn Unit> = Box::new(Yen(1)); // error[E0038]: the trait `Unit` is not dyn compatible
}

// ---------------------------------------------------------------------------
// 5. 関連型と標準 trait
// ---------------------------------------------------------------------------

/// n, n-1, …, 1 を返すイテレータ
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32; // 関連型: 「この Iterator が返す要素の型」を実装側が 1 つに決める

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

fn a5_iterator() {
    // next を 1 つ書くだけで、map / filter / sum / collect など 70 以上のメソッドが使える
    let v: Vec<u32> = Countdown(5).collect();
    let even_sum: u32 = Countdown(10).filter(|n| n % 2 == 0).sum();
    println!("Countdown(5) = {v:?}, 偶数の和 = {even_sum}");
    for n in Countdown(3) {
        print!("{n}… ");
    }
    println!("発射");
}

/// derive で標準 trait の実装を自動生成
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct AccountCode(u16);

#[derive(Debug, Clone, PartialEq, Default)]
struct Config {
    retries: u32,
    verbose: bool,
}

/// Display は derive できない（ユーザー向けの表示は自分で決める）
impl fmt::Display for AccountCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:04}", self.0)
    }
}

fn a5_derive() {
    let a = AccountCode(101);
    let b = a; // Copy なので a はまだ使える（S1 の move が起きない）
    println!("Debug = {a:?}, Display = {b}");
    println!("a == b: {}, a < AccountCode(401): {}", a == b, a < AccountCode(401));

    let mut codes = vec![AccountCode(401), AccountCode(101), AccountCode(201)];
    codes.sort(); // Ord があるのでソートできる
    println!("sorted = {codes:?}");

    let mut names = HashMap::new(); // Eq + Hash があるので HashMap のキーにできる
    names.insert(AccountCode(101), "現金");
    println!("names[#0101] = {:?}", names.get(&AccountCode(101)));

    // Default: 一部のフィールドだけ指定して残りはデフォルト値
    let cfg = Config {
        retries: 3,
        ..Default::default()
    };
    println!("{cfg:?} → retries = {}, verbose = {}", cfg.retries, cfg.verbose);
    println!("AccountCode::default() = {:?}", AccountCode::default());
}

struct Celsius(f64);
struct Fahrenheit(f64);

// From を実装すると Into が自動で付く（ブランケット実装）
impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

fn a5_from_into() {
    let f1 = Fahrenheit::from(Celsius(100.0));
    let f2: Fahrenheit = Celsius(-40.0).into(); // into は代入先の型から変換先を推論
    println!("100℃ = {}℉, -40℃ = {}℉", f1.0, f2.0);

    // 標準ライブラリにも多数ある
    let s: String = "abc".into();
    let big: i64 = i64::from(7_i32); // 情報が失われない変換は From
    let small = u8::try_from(300_i32); // 失敗し得る変換は TryFrom → Result
    println!("{s:?}, {big}, {small:?}");
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

// 演算子オーバーロード: a + b は Add::add(a, b) の糖衣構文
impl Add for Vec2 {
    type Output = Vec2; // 足した結果の型（関連型）
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

fn a5_operator() {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 0.5, y: -1.0 };
    println!("{a:?} + {b:?} = {:?}", a + b);
}

// ---------------------------------------------------------------------------
// 6. orphan rule と newtype
// ---------------------------------------------------------------------------

// 外部の trait（Display）を外部の型（Vec）に実装することはできない
// impl fmt::Display for Vec<String> { ... }
// error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate

/// newtype: 外部の型を自分の型で包めば、外部の trait を実装できる
struct Lines(Vec<String>);

impl fmt::Display for Lines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, line) in self.0.iter().enumerate() {
            writeln!(f, "{:>3}| {line}", i + 1)?;
        }
        Ok(())
    }
}

fn o6_newtype() {
    let lines = Lines(vec!["fn main() {".into(), "    println!(\"hi\");".into(), "}".into()]);
    print!("{lines}");

    // 自分の trait なら外部の型にも実装できる（overview の 21_i32.double() と同じ）
    trait Double {
        fn double(&self) -> Self;
    }
    impl Double for i32 {
        fn double(&self) -> Self {
            self * 2
        }
    }
    println!("21.double() = {}", 21_i32.double());
}

// ---------------------------------------------------------------------------
// 8. 設計の指針: 金額型と試算表
// ---------------------------------------------------------------------------

/// 金額（円）。i64 をそのまま使わず newtype にすると、数量や ID と取り違えない
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Money(i64);

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}

// Sum を実装すると iter().sum::<Money>() が使える
impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::default(), |acc, m| acc + m)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        iter.copied().sum()
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.0.unsigned_abs().to_string();
        let mut out = String::new();
        if self.0 < 0 {
            out.push('-');
        }
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        out.push('円');
        // write! ではなく pad を使うと、呼び出し側の {:>10} などの幅指定が効く
        f.pad(&out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Account {
    Cash,
    Purchases,
    AccountsPayable,
    Sales,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Account::Cash => "現金",
            Account::Purchases => "仕入",
            Account::AccountsPayable => "買掛金",
            Account::Sales => "売上",
        };
        f.pad(name)
    }
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Copy)]
struct Line {
    account: Account,
    side: Side,
    amount: Money,
}

/// 試算表の 1 行: (勘定科目, 借方合計, 貸方合計)
type Row = (Account, Money, Money);

/// 仕訳行の集まりから試算表を作る。Vec でもスライスでも、イテレータでも受けられる
fn trial_balance<'a, I>(lines: I) -> Vec<Row>
where
    I: IntoIterator<Item = &'a Line>,
{
    // BTreeMap は Ord 順に並ぶ → Account の宣言順で出力される
    let mut map: BTreeMap<Account, (Money, Money)> = BTreeMap::new();
    for l in lines {
        let e = map.entry(l.account).or_default();
        match l.side {
            Side::Debit => e.0 += l.amount,
            Side::Credit => e.1 += l.amount,
        }
    }
    map.into_iter().map(|(a, (d, c))| (a, d, c)).collect()
}

/// 出力形式。実行時に選びたいので dyn で使う
trait Render {
    fn render(&self, rows: &[Row]) -> String;
}

struct TextRender;
struct CsvRender;

impl Render for TextRender {
    fn render(&self, rows: &[Row]) -> String {
        let mut s = format!("{:>12} {:>12}  科目\n", "借方", "貸方");
        for (a, d, c) in rows {
            s += &format!("{d:>12} {c:>12}  {a}\n");
        }
        let d: Money = rows.iter().map(|r| r.1).sum();
        let c: Money = rows.iter().map(|r| r.2).sum();
        s += &format!("{d:>12} {c:>12}  合計\n");
        s
    }
}

impl Render for CsvRender {
    fn render(&self, rows: &[Row]) -> String {
        let mut s = String::from("account,debit,credit\n");
        for (a, d, c) in rows {
            s += &format!("{a},{},{}\n", d.0, c.0);
        }
        s
    }
}

fn pick_renderer(format: &str) -> Box<dyn Render> {
    match format {
        "csv" => Box::new(CsvRender),
        _ => Box::new(TextRender),
    }
}

fn d8_trial_balance() {
    use Account::*;
    use Side::*;
    let line = |account, side, yen| Line {
        account,
        side,
        amount: Money(yen),
    };
    let lines = [
        // 商品を掛けで仕入れた
        line(Purchases, Debit, 30_000),
        line(AccountsPayable, Credit, 30_000),
        // 商品を現金で売り上げた
        line(Cash, Debit, 52_000),
        line(Sales, Credit, 52_000),
        // 買掛金の一部を現金で支払った
        line(AccountsPayable, Debit, 10_000),
        line(Cash, Credit, 10_000),
    ];

    let rows = trial_balance(&lines);
    for format in ["text", "csv"] {
        println!("-- {format} --");
        print!("{}", pick_renderer(format).render(&rows));
    }

    // 1 つの Money を足し合わせる操作が、型と trait だけで表現できている
    let cash_in: Money = lines
        .iter()
        .filter(|l| matches!((l.account, l.side), (Cash, Debit)))
        .map(|l| l.amount)
        .sum();
    println!("現金の入金合計 = {cash_in}");
    // Money(1) + 1; // error[E0308]: mismatched types（素の i64 とは足せない）
}

// ---------------------------------------------------------------------------
// テスト（cargo test -p traits）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_works_for_many_types() {
        assert_eq!(largest(&[3, 7, 2]), Some(7));
        assert_eq!(largest(&['a', 'z']), Some('z'));
        assert_eq!(largest::<f64>(&[]), None);
    }

    #[test]
    fn pair_methods() {
        let p = Pair::new(1, 2).swap();
        assert_eq!((p.a, p.b), (2, 1));
        assert_eq!(Pair::new(1.0, 4.0).mean(), 2.5);
        assert_eq!(Pair::new(3, 8).show_larger(), "b = 8");
    }

    #[test]
    fn dyn_and_static_agree() {
        let rects = [Rect { w: 1.0, h: 2.0 }, Rect { w: 3.0, h: 4.0 }];
        let boxed: Vec<Box<dyn Shape>> = vec![
            Box::new(Rect { w: 1.0, h: 2.0 }),
            Box::new(Rect { w: 3.0, h: 4.0 }),
        ];
        assert_eq!(total_area_static(&rects), total_area_dyn(&boxed));
    }

    #[test]
    fn countdown_iterator() {
        assert_eq!(Countdown(3).collect::<Vec<_>>(), vec![3, 2, 1]);
        assert_eq!(Countdown(0).next(), None);
    }

    #[test]
    fn conversions() {
        let f: Fahrenheit = Celsius(100.0).into();
        assert_eq!(f.0, 212.0);
        assert!(u8::try_from(300_i32).is_err());
    }

    #[test]
    fn money_display_and_sum() {
        assert_eq!(Money(1_234_567).to_string(), "1,234,567円");
        assert_eq!(Money(-500).to_string(), "-500円");
        assert_eq!(format!("{:>8}", Money(500)), "    500円");
        let total: Money = [Money(100), Money(250)].iter().sum();
        assert_eq!(total, Money(350));
    }

    #[test]
    fn trial_balance_is_balanced() {
        let lines = [
            Line {
                account: Account::Cash,
                side: Side::Debit,
                amount: Money(500),
            },
            Line {
                account: Account::Sales,
                side: Side::Credit,
                amount: Money(500),
            },
        ];
        let rows = trial_balance(&lines);
        let d: Money = rows.iter().map(|r| r.1).sum();
        let c: Money = rows.iter().map(|r| r.2).sum();
        assert_eq!(d, c);
        assert_eq!(rows[0].0, Account::Cash); // Ord 順
    }
}
