//! `notes/rust-error-handling.md` のサンプルを実際に動かすための crate。
//!
//! 実行: `cargo run -p errors`
//! テスト: `cargo test -p errors`（panic の例はテスト側にある）
//!
//! `// error[...]` と書いてある行はコンパイルエラーになる例。
//! コメントを外して `cargo build -p errors` すると、そのエラー番号が出ることを確かめられる。

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// main も Result を返せる。Err で終わると Debug 表示のエラーを出して終了コード 1 になる
fn main() -> Result<(), Box<dyn Error>> {
    println!("== 0. 失敗の 3 分類 ==");
    s0_kinds();

    println!("\n== 1. Option<T> ==");
    o1_match();
    o1_if_let_else();
    o1_combinators();
    o1_question();

    println!("\n== 2. Result<T, E> ==");
    r2_match();
    r2_combinators();
    r2_convert();
    r2_collect();

    println!("\n== 3. ? 演算子 ==");
    q3_desugar();
    q3_box_dyn()?;

    println!("\n== 4. 独自エラー型 ==");
    e4_custom();
    e4_source_chain();
    e4_box_dyn();

    println!("\n== 5. panic と unwrap / expect ==");
    p5_panic();

    println!("\n== 7. 設計の指針: 仕訳の貸借一致 ==");
    d7_journal();

    Ok(())
}

// ---------------------------------------------------------------------------
// 0. 失敗の 3 分類
// ---------------------------------------------------------------------------

fn s0_kinds() {
    let v = [10, 20, 30];

    // 値の不在 → Option
    println!("v.get(1) = {:?}, v.get(9) = {:?}", v.get(1), v.get(9));

    // 回復可能な失敗 → Result
    println!("\"42\".parse::<i32>()  = {:?}", "42".parse::<i32>());
    println!("\"abc\".parse::<i32>() = {:?}", "abc".parse::<i32>());

    // 回復不能（バグ） → panic
    // let i = 9; let x = v[i]; // 実行時 panic: index out of bounds（コンパイルは通る）
}

// ---------------------------------------------------------------------------
// 1. Option<T>
// ---------------------------------------------------------------------------

fn find_user(id: u32) -> Option<&'static str> {
    match id {
        1 => Some("alice"),
        2 => Some("bob"),
        3 => Some(""), // 名前が空のユーザー（o1_question で使う）
        _ => None,
    }
}

fn o1_match() {
    for id in [1, 9] {
        match find_user(id) {
            Some(name) => println!("id={id}: {name}"),
            None => println!("id={id}: 見つからない"),
        }
    }
    // let name: &str = find_user(1); // error[E0308]: mismatched types（Option<&str> は &str ではない）

    // match find_user(1) {
    //     Some(name) => println!("{name}"),
    // } // error[E0004]: non-exhaustive patterns: `None` not covered
}

fn greet(id: u32) -> String {
    // let-else: パターンに合わなければ else ブロックで脱出する（else は必ず return / break 等で抜ける）
    let Some(name) = find_user(id) else {
        return String::from("こんにちは、ゲストさん");
    };
    format!("こんにちは、{name}さん")
}

fn o1_if_let_else() {
    // None のときに何もしないなら if let
    if let Some(name) = find_user(2) {
        println!("if let: {name}");
    }
    println!("let-else: {} / {}", greet(1), greet(9));
}

fn o1_combinators() {
    let len = find_user(1).map(|s| s.len()); // Some(5)
    let upper = find_user(9).map(|s| s.to_uppercase()); // None のまま
    let initial = find_user(2).and_then(|s| s.chars().next()); // クロージャ自体が Option を返す
    let name = find_user(9).unwrap_or("guest"); // None ならデフォルト値
    let r = find_user(9).ok_or("no such user"); // Option → Result
    println!("map = {len:?}, map(None) = {upper:?}, and_then = {initial:?}");
    println!("unwrap_or = {name}, ok_or = {r:?}");
}

/// ユーザー名の頭文字を大文字で返す。ユーザーがいない・名前が空なら None
fn initial_upper(id: u32) -> Option<char> {
    let name = find_user(id)?; // None ならここで return None
    let c = name.chars().next()?; // 空文字列でも None
    Some(c.to_ascii_uppercase())
}

fn o1_question() {
    println!(
        "initial_upper: 1 → {:?}, 3 → {:?}, 9 → {:?}",
        initial_upper(1),
        initial_upper(3),
        initial_upper(9)
    );
}

// ---------------------------------------------------------------------------
// 2. Result<T, E>
// ---------------------------------------------------------------------------

fn parse_age(s: &str) -> Result<u8, ParseIntError> {
    s.trim().parse::<u8>()
}

fn r2_match() {
    for s in ["30", "abc", "300"] {
        match parse_age(s) {
            Ok(age) => println!("{s:?} → Ok({age})"),
            Err(e) => println!("{s:?} → Err({e})"),
        }
    }
    // parse_age("1"); // warning: unused `Result` that must be used（Result は #[must_use]）
}

fn r2_combinators() {
    let doubled = parse_age("21").map(|n| n * 2); // Ok の中身だけ変換
    let msg = parse_age("x").map_err(|e| format!("年齢が不正: {e}")); // Err の中身だけ変換
    println!("map = {doubled:?}, map_err = {msg:?}");

    // and_then: 「失敗しうる処理」を続けてつなぐ。エラー型は揃える必要がある
    let checked: Result<u8, String> = parse_age("140").map_err(|e| e.to_string()).and_then(|n| {
        if n <= 130 {
            Ok(n)
        } else {
            Err(format!("{n} 歳は範囲外"))
        }
    });
    println!("and_then = {checked:?}");

    // unwrap_or_else: Err のときだけクロージャを実行してデフォルト値を作る
    let age = parse_age("x").unwrap_or_else(|e| {
        println!("  fallback（{e}）→ 0 を使う");
        0
    });
    println!("unwrap_or_else = {age}");
}

fn r2_convert() {
    let a: Option<u8> = parse_age("x").ok(); // Result → Option（エラーの中身は捨てる）
    let b: Result<&str, &str> = find_user(9).ok_or("no such user"); // Option → Result
    println!("ok() = {a:?}, ok_or = {b:?}");
}

fn r2_collect() {
    // Iterator<Item = Result<T, E>> は Result<Vec<T>, E> に collect できる。最初の Err で止まる
    let good: Result<Vec<u8>, _> = ["1", "2", "3"].into_iter().map(parse_age).collect();
    let bad: Result<Vec<u8>, _> = ["1", "x", "3"].into_iter().map(parse_age).collect();
    println!("collect(good) = {good:?}");
    println!("collect(bad)  = {bad:?}");

    // 失敗した要素を読み飛ばすなら filter_map + ok()
    let sum: u32 = ["1", "x", "3"]
        .into_iter()
        .filter_map(|s| parse_age(s).ok())
        .map(u32::from)
        .sum();
    println!("filter_map で読み飛ばして合計 = {sum}");
}

// ---------------------------------------------------------------------------
// 3. ? 演算子
// ---------------------------------------------------------------------------

// ? を手で展開するとこうなる（clippy に「? を使え」と言われるのを黙らせている）
#[allow(clippy::question_mark, clippy::useless_conversion)]
fn sum_two_manual(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = match a.trim().parse() {
        Ok(v) => v,
        Err(e) => return Err(From::from(e)), // ← From による変換が挟まる
    };
    let y: i32 = match b.trim().parse() {
        Ok(v) => v,
        Err(e) => return Err(From::from(e)),
    };
    Ok(x + y)
}

fn sum_two(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.trim().parse()?;
    let y: i32 = b.trim().parse()?;
    Ok(x + y)
}

// fn no_result() {
//     let n: i32 = "1".parse()?; // error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
//     println!("{n}");
// }

// fn mixed() -> Option<i32> {
//     let n: i32 = "1".parse()?; // error[E0277]: the `?` operator can only be used on `Option`s, not `Result`s, in a function that returns `Option`
//     Some(n)
// }

fn q3_desugar() {
    println!(
        "手で展開: {:?} / {:?}",
        sum_two_manual("1", "2"),
        sum_two_manual("1", "x")
    );
    println!(
        "? を使う: {:?} / {:?}",
        sum_two("1", "2"),
        sum_two("1", "x")
    );
}

// 型の違うエラーも Box<dyn Error> にまとめて ? で返せる
fn parse_pair(a: &str, b: &str) -> Result<(i32, f64), Box<dyn Error>> {
    let i: i32 = a.parse()?; // ParseIntError → Box<dyn Error>
    let f: f64 = b.parse()?; // ParseFloatError → Box<dyn Error>
    Ok((i, f))
}

fn q3_box_dyn() -> Result<(), Box<dyn Error>> {
    match parse_pair("1", "abc") {
        Ok(p) => println!("parse_pair = {p:?}"),
        Err(e) => println!("parse_pair エラー: {e}"),
    }
    let (i, f) = parse_pair("3", "0.5")?; // Err なら main まで伝播して終了する
    println!("parse_pair = ({i}, {f})");
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. 独自エラー型（README 課題 4「簡易電卓」の芯）
// ---------------------------------------------------------------------------

/// 電卓で起こりうる失敗を列挙した型
#[derive(Debug, PartialEq)]
enum CalcError {
    Parse(ParseIntError), // 元のエラーを包む
    UnknownOp(String),
    DivByZero,
    Overflow,
    WrongArity(usize),
}

// ユーザー向けの文言。原因（source）の文言はここに重ねて書かない
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::Parse(_) => write!(f, "数値として読めない"),
            CalcError::UnknownOp(op) => write!(f, "未知の演算子: {op}"),
            CalcError::DivByZero => write!(f, "0 で割ろうとした"),
            CalcError::Overflow => write!(f, "計算結果が i64 の範囲を超えた"),
            CalcError::WrongArity(n) => {
                write!(f, "「数 演算子 数」の 3 要素が必要（{n} 要素だった）")
            }
        }
    }
}

// Debug + Display を実装していれば、Error trait は source 以外書かなくてよい
impl Error for CalcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CalcError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

// これがあるので calc の中で ParseIntError に ? を使える
impl From<ParseIntError> for CalcError {
    fn from(e: ParseIntError) -> Self {
        CalcError::Parse(e)
    }
}

/// "12 + 3" のような「数 演算子 数」の式を計算する
fn calc(expr: &str) -> Result<i64, CalcError> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    let [a, op, b] = tokens[..] else {
        return Err(CalcError::WrongArity(tokens.len()));
    };
    let a: i64 = a.parse()?; // ParseIntError → CalcError::Parse（From で自動変換）
    let b: i64 = b.parse()?;
    match op {
        "+" => a.checked_add(b).ok_or(CalcError::Overflow),
        "-" => a.checked_sub(b).ok_or(CalcError::Overflow),
        "*" => a.checked_mul(b).ok_or(CalcError::Overflow),
        "/" if b == 0 => Err(CalcError::DivByZero),
        "/" => a.checked_div(b).ok_or(CalcError::Overflow), // i64::MIN / -1 が Overflow
        _ => Err(CalcError::UnknownOp(op.to_string())),
    }
}

fn e4_custom() {
    let exprs = [
        "12 + 3",
        "7 / 0",
        "1 ^ 2",
        "x * 2",
        "1 +",
        "9223372036854775807 + 1",
    ];
    for expr in exprs {
        match calc(expr) {
            Ok(v) => println!("{expr:?} = {v}"),
            Err(e) => println!("{expr:?} → {e}"),
        }
    }
}

/// エラーと、source() でたどれる原因をすべて表示する
fn print_chain(e: &dyn Error) {
    println!("error: {e}");
    let mut cur = e.source();
    while let Some(cause) = cur {
        println!("  caused by: {cause}");
        cur = cause.source();
    }
}

fn e4_source_chain() {
    if let Err(e) = calc("x * 2") {
        print_chain(&e);
    }
}

fn run_line(expr: &str) -> Result<i64, Box<dyn Error>> {
    let v = calc(expr)?; // CalcError → Box<dyn Error>（Error を実装していれば自動）
    Ok(v)
}

fn e4_box_dyn() {
    if let Err(e) = run_line("7 / 0") {
        // Box<dyn Error> にすると具体的な型は見えなくなる。取り出すなら downcast_ref
        match e.downcast_ref::<CalcError>() {
            Some(CalcError::DivByZero) => println!("downcast_ref: 0 除算だと判別できた"),
            _ => println!("downcast_ref: その他のエラー: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. panic と unwrap / expect
// ---------------------------------------------------------------------------

fn p5_panic() {
    let v = [1, 2, 3];
    // expect のメッセージには「なぜ失敗しないと言えるか」を書く
    let first = v.first().expect("v は 3 要素の配列なので空ではない");
    println!("expect: first = {first}");

    // let n: i32 = "abc".parse().unwrap(); // 実行時 panic: called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
    // panic!("ここには来ないはず");          // 実行時 panic: ここには来ないはず
    println!("panic する例はコメントアウトとテスト（cargo test -p errors）にある");
}

// ---------------------------------------------------------------------------
// 7. 設計の指針: 仕訳の貸借一致をエラー型で守る（ステージ2 のスケッチ）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Side {
    Debit,  // 借方
    Credit, // 貸方
}

#[derive(Debug, PartialEq)]
enum JournalError {
    Empty,
    ZeroAmount,
    Unbalanced { debit: u64, credit: u64 },
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JournalError::Empty => write!(f, "仕訳に行が 1 つもない"),
            JournalError::ZeroAmount => write!(f, "金額 0 の行がある"),
            JournalError::Unbalanced { debit, credit } => {
                write!(f, "貸借が一致しない（借方 {debit} / 貸方 {credit}）")
            }
        }
    }
}

impl Error for JournalError {}

/// 貸借が一致した仕訳。new を通らないと作れないので、存在する時点で一致が保証される
#[derive(Debug)]
struct Journal {
    lines: Vec<(Side, u64)>, // 金額は円単位の整数（浮動小数の誤差を避ける）
}

impl Journal {
    fn new(lines: Vec<(Side, u64)>) -> Result<Self, JournalError> {
        if lines.is_empty() {
            return Err(JournalError::Empty);
        }
        if lines.iter().any(|&(_, amount)| amount == 0) {
            return Err(JournalError::ZeroAmount);
        }
        let (debit, credit) = lines
            .iter()
            .fold((0, 0), |(d, c), &(side, amount)| match side {
                Side::Debit => (d + amount, c),
                Side::Credit => (d, c + amount),
            });
        if debit != credit {
            return Err(JournalError::Unbalanced { debit, credit });
        }
        Ok(Journal { lines })
    }

    fn total(&self) -> u64 {
        self.lines
            .iter()
            .filter(|(side, _)| matches!(side, Side::Debit))
            .map(|&(_, amount)| amount)
            .sum()
    }
}

fn d7_journal() {
    let ok = Journal::new(vec![(Side::Debit, 1000), (Side::Credit, 1000)]);
    let ng = Journal::new(vec![(Side::Debit, 1000), (Side::Credit, 900)]);
    match ok {
        Ok(j) => println!("仕訳 OK: 合計 {} 円", j.total()),
        Err(e) => println!("仕訳 NG: {e}"),
    }
    match ng {
        Ok(j) => println!("仕訳 OK: 合計 {} 円", j.total()),
        Err(e) => println!("仕訳 NG: {e}"),
    }
}

// ---------------------------------------------------------------------------
// テスト（cargo test -p errors）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_ok() {
        assert_eq!(calc("12 + 3"), Ok(15));
        assert_eq!(calc("  6   *  7 "), Ok(42));
    }

    #[test]
    fn calc_errors() {
        assert_eq!(calc("7 / 0"), Err(CalcError::DivByZero));
        assert_eq!(calc("1 +"), Err(CalcError::WrongArity(2)));
        assert_eq!(
            calc(&format!("{} / -1", i64::MIN)),
            Err(CalcError::Overflow)
        );
        // 中身まで比べたくないときは matches!
        assert!(matches!(calc("x * 2"), Err(CalcError::Parse(_))));
    }

    // テスト関数も Result を返せる。? で失敗させられる
    #[test]
    fn question_in_test() -> Result<(), CalcError> {
        let v = calc("2 * 21")?;
        assert_eq!(v, 42);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "InvalidDigit")]
    fn unwrap_panics() {
        let _n: i32 = "abc".parse().unwrap();
    }

    #[test]
    fn journal_invariant() {
        assert!(Journal::new(vec![(Side::Debit, 500), (Side::Credit, 500)]).is_ok());
        assert_eq!(Journal::new(vec![]).unwrap_err(), JournalError::Empty);
        assert_eq!(
            Journal::new(vec![(Side::Debit, 500), (Side::Credit, 300)]).unwrap_err(),
            JournalError::Unbalanced {
                debit: 500,
                credit: 300
            }
        );
    }
}
