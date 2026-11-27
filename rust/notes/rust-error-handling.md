# エラーハンドリング — `Option` / `Result` / `?`（S4 深掘り）

> `rust-syntax-overview.md` の S4 を詳しく掘り下げたリファレンス。
> サンプルコードは `exercises/errors/src/main.rs` に関数単位でまとめてあり、`cargo run -p errors` でそのまま動く（panic の例は `cargo test -p errors`）。
> エラーになる例はコメントアウトしてあるので、外してエラーメッセージを実際に読むこと。
> マトリクス（`rust-3layer-matrix.md`）の S4 には**このファイルを写さず**、自分の言葉で書くこと。

---

## 目次

0. [前提: 失敗の 3 分類](#0-前提-失敗の-3-分類)
1. [`Option<T>` — 値が無いかもしれない](#1-optiont--値が無いかもしれない)
2. [`Result<T, E>` — 失敗するかもしれない](#2-resultt-e--失敗するかもしれない)
3. [`?` 演算子 — エラーの伝播](#3--演算子--エラーの伝播)
4. [独自エラー型](#4-独自エラー型)
5. [`panic!` と `unwrap` / `expect`](#5-panic-と-unwrap--expect)
6. [よくあるコンパイルエラーと直し方](#6-よくあるコンパイルエラーと直し方)
7. [設計の指針](#7-設計の指針)
8. [理解度チェック](#8-理解度チェック)

---

## 0. 前提: 失敗の 3 分類

Rust は「うまくいかない状況」を性質ごとに別の仕組みで扱う。

| 状況 | 例 | Rust の道具 | JS での表し方 |
|---|---|---|---|
| 値が**無い**ことがある（異常ではない） | 配列の範囲外の `get`、検索で見つからない | `Option<T>` | `null` / `undefined` を返す |
| **失敗**することがあり、呼び出し元が対処できる | 文字列 → 数値の変換、ファイルが無い | `Result<T, E>` | `throw` して `try-catch` |
| 起きてはいけない（**バグ**） | 添字の範囲外アクセス、不変条件の破れ | `panic!` | `throw`（区別しない） |

```rust
let v = [10, 20, 30];
v.get(9);                 // None（無いだけ）
"abc".parse::<i32>();     // Err(ParseIntError { kind: InvalidDigit })（失敗）
v[i];                     // i が範囲外なら panic（バグ）
```

**JS との違い**: JS では「無い」も「失敗」も「バグ」も `null` か `throw` に混ざり、しかも**関数のシグネチャに現れない**。Rust では戻り値の型を見れば、その関数が「無いことがある」のか「失敗することがある」のかが分かり、**扱い忘れるとコンパイルが通らない**。

---

## 1. `Option<T>` — 値が無いかもしれない

`Option` はただの `enum`。言語に特別扱いされているわけではない。

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`Option<&str>` と `&str` は**別の型**なので、中身を取り出す処理を書かない限り `&str` として使えない。これが「null 参照」を型で防ぐ仕組み。

### 取り出し方 1: `match`

```rust
fn find_user(id: u32) -> Option<&'static str> {
    match id {
        1 => Some("alice"),
        2 => Some("bob"),
        _ => None,
    }
}

match find_user(id) {
    Some(name) => println!("id={id}: {name}"),
    None => println!("id={id}: 見つからない"),
}

// let name: &str = find_user(1); // error[E0308]: mismatched types
```

`match` は網羅性が検査されるので、`None` の腕を書き忘れると E0004 になる。

### 取り出し方 2: `if let` と `let ... else`

```rust
// None のときに何もしないなら if let
if let Some(name) = find_user(2) {
    println!("{name}");
}

// None なら早期 return し、以降は中身を普通の変数として使う
fn greet(id: u32) -> String {
    let Some(name) = find_user(id) else {
        return String::from("こんにちは、ゲストさん");
    };
    format!("こんにちは、{name}さん")
}
```

`let ... else` の `else` ブロックは**必ず脱出しなければならない**（`return` / `break` / `continue` / `panic!`）。抜けないと後続の `name` に値が入らないため。ネストを浅く保てるので、関数冒頭のガード節に向いている。

### 取り出し方 3: コンビネータ（メソッド）

| メソッド | `Some(x)` のとき | `None` のとき | 用途 |
|---|---|---|---|
| `map(f)` | `Some(f(x))` | `None` | 中身だけ変換 |
| `and_then(f)` | `f(x)`（`f` が `Option` を返す） | `None` | 失敗しうる処理をつなぐ |
| `unwrap_or(d)` | `x` | `d` | デフォルト値 |
| `unwrap_or_else(f)` | `x` | `f()` | デフォルト値の計算が重いとき |
| `unwrap_or_default()` | `x` | `T::default()` | `0` / `""` / 空の `Vec` など |
| `ok_or(e)` | `Ok(x)` | `Err(e)` | `Result` へ変換 |
| `filter(p)` | `p(&x)` なら `Some(x)` | `None` | 条件を満たさなければ捨てる |

```rust
find_user(1).map(|s| s.len());               // Some(5)
find_user(9).map(|s| s.to_uppercase());      // None のまま
find_user(2).and_then(|s| s.chars().next()); // Some('b')
find_user(9).unwrap_or("guest");             // "guest"
find_user(9).ok_or("no such user");          // Err("no such user")
```

`map` と `and_then` の違いは、クロージャが `T` を返すか `Option<T>` を返すか。`map` に `Option` を返すクロージャを渡すと `Option<Option<T>>` になる。

`unwrap_or(f())` と書くと `f()` は `Some` のときも**必ず評価される**（引数は呼び出し前に評価されるため）。重い処理なら `unwrap_or_else(|| f())` にする。`ok_or` と `ok_or_else` の関係も同じ。

### `Option` に対する `?`

```rust
/// ユーザー名の頭文字を大文字で返す。ユーザーがいない・名前が空なら None
fn initial_upper(id: u32) -> Option<char> {
    let name = find_user(id)?;    // None ならここで return None
    let c = name.chars().next()?; // 空文字列でも None
    Some(c.to_ascii_uppercase())
}
```

`?` は `Option` を返す関数の中でも使える。詳しくは [3 節](#3--演算子--エラーの伝播)。

**JS との違い**: JS のオプショナルチェーン `user?.name?.[0]` は見た目が似ているが、結果が `undefined` になるだけで**使う側に確認を強制しない**。Rust の `?` は関数の戻り値型を `Option` にしない限り書けず、呼び出し元は `None` を扱わない限り中身を使えない。

---

## 2. `Result<T, E>` — 失敗するかもしれない

これもただの `enum`。

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`E` には「なぜ失敗したか」を表す型が入る。`Option` と違って**失敗の理由を運べる**。

### `match` で分岐する

```rust
use std::num::ParseIntError;

fn parse_age(s: &str) -> Result<u8, ParseIntError> {
    s.trim().parse::<u8>()
}

for s in ["30", "abc", "300"] {
    match parse_age(s) {
        Ok(age) => println!("{s:?} → Ok({age})"),
        Err(e) => println!("{s:?} → Err({e})"),
    }
}
// "30"  → Ok(30)
// "abc" → Err(invalid digit found in string)
// "300" → Err(number too large to fit in target type)   ← u8 は 0〜255
```

`Result` には `#[must_use]` が付いているので、戻り値を捨てると警告が出る。

```rust
parse_age("1"); // warning: unused `Result` that must be used
```

どうしても捨てたいときは `let _ = parse_age("1");` と書いて「わざと捨てた」ことを明示する。

### コンビネータ

`Option` とほぼ同じ名前のメソッドがある。

| メソッド | `Ok(x)` のとき | `Err(e)` のとき | 用途 |
|---|---|---|---|
| `map(f)` | `Ok(f(x))` | `Err(e)` | 成功値だけ変換 |
| `map_err(f)` | `Ok(x)` | `Err(f(e))` | エラーだけ変換（型を揃える） |
| `and_then(f)` | `f(x)`（`f` が `Result` を返す） | `Err(e)` | 失敗しうる処理をつなぐ |
| `unwrap_or(d)` / `unwrap_or_else(f)` | `x` | `d` / `f(e)` | デフォルト値 |
| `ok()` | `Some(x)` | `None` | `Option` へ変換（エラー内容は捨てる） |
| `is_ok()` / `is_err()` | | | 判定だけしたいとき |

```rust
parse_age("21").map(|n| n * 2);                         // Ok(42)
parse_age("x").map_err(|e| format!("年齢が不正: {e}")); // Err("年齢が不正: invalid digit ...")

// and_then はエラー型を揃える必要がある。map_err で String に寄せてからつなぐ
let checked: Result<u8, String> = parse_age("140")
    .map_err(|e| e.to_string())
    .and_then(|n| if n <= 130 { Ok(n) } else { Err(format!("{n} 歳は範囲外")) });
// Err("140 歳は範囲外")
```

### `Option` ⇔ `Result` の変換

```rust
let a: Option<u8> = parse_age("x").ok();                         // Result → Option
let b: Result<&str, &str> = find_user(9).ok_or("no such user");  // Option → Result
```

「関数の中で `Option` と `Result` が混ざる」ときは、どちらかに寄せてから `?` を使う（[6 節](#6-よくあるコンパイルエラーと直し方)の E0277）。

### `collect` で `Result` をまとめる

`Iterator<Item = Result<T, E>>` は `Result<Vec<T>, E>` に `collect` できる。全部成功なら `Ok(Vec)`、**最初の `Err` で止まって**それを返す。

```rust
let good: Result<Vec<u8>, _> = ["1", "2", "3"].into_iter().map(parse_age).collect();
// Ok([1, 2, 3])
let bad: Result<Vec<u8>, _> = ["1", "x", "3"].into_iter().map(parse_age).collect();
// Err(ParseIntError { kind: InvalidDigit })

// 失敗した要素を読み飛ばすなら filter_map + ok()
let sum: u32 = ["1", "x", "3"].into_iter()
    .filter_map(|s| parse_age(s).ok())
    .map(u32::from)
    .sum();  // 4
```

CSV の各行をパースする（README 課題 7）ときに、「1 行でも壊れていたら全体を失敗にする」なら `collect`、「壊れた行は飛ばす」なら `filter_map` を選ぶ。

**JS との違い**: `JSON.parse` や `Number()` が失敗するかどうかは、ドキュメントを読むか実行するまで分からない（`Number("x")` は例外ではなく `NaN` を返す）。Rust では `parse` の戻り値が `Result` なので、型を見ただけで失敗しうると分かる。

---

## 3. `?` 演算子 — エラーの伝播

### `?` は何の省略か

```rust
fn sum_two(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.trim().parse()?;
    let y: i32 = b.trim().parse()?;
    Ok(x + y)
}
```

`expr?` はおおよそ次の `match` に展開される。

```rust
match expr {
    Ok(v) => v,                          // 成功なら中身を取り出して処理を続ける
    Err(e) => return Err(From::from(e)), // 失敗なら From で変換して即 return
}
```

ポイントは 2 つ:

1. **早期 return** する。`?` が付いた式で失敗すると、関数の残りは実行されない
2. **`From::from` でエラー型を変換**する。関数の戻り値のエラー型 `E2` に `From<E1>` が実装されていれば、`E1` のエラーにも `?` が使える（[4 節](#4-独自エラー型)で使う）

`Option` に対する `?` は `None => return None` になる。

### `?` が使える場所

`?` は**その関数の戻り値型**に合わせて return するので、戻り値が `Result`（または `Option`）の関数の中でしか使えない。

```rust
fn no_result() {
    let n: i32 = "1".parse()?;
    // error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
}
```

`main` も `Result` を返せる。`Err` で終わると、エラーの **Debug 表示**が出力されて終了コード 1 になる。

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (i, f) = parse_pair("3", "0.5")?;
    println!("({i}, {f})");
    Ok(())
}
```

### `Box<dyn Error>` — 何でも入るエラー型

`Box<dyn Error>` は「`std::error::Error` trait を実装した何らかの型」をヒープに置いたもの。標準ライブラリのエラー型はすべて `Error` を実装していて、`Box<dyn Error>` への `From` 変換も用意されているので、**種類の違うエラーをまとめて `?` で返せる**。

```rust
fn parse_pair(a: &str, b: &str) -> Result<(i32, f64), Box<dyn Error>> {
    let i: i32 = a.parse()?; // ParseIntError   → Box<dyn Error>
    let f: f64 = b.parse()?; // ParseFloatError → Box<dyn Error>
    Ok((i, f))
}
```

`dyn Trait`（trait オブジェクト）の仕組みは S5 の範囲。ここでは「どんなエラーでも入る箱」と理解しておけばよい。

**JS との違い**: JS の `throw` は呼び出し階層を**暗黙に**突き抜けていく。どの関数が例外を投げうるかは、コードを全部読まないと分からない。Rust の `?` は伝播させる箇所すべてに `?` が目に見えて付いており、伝播できるのは戻り値が `Result` の関数だけ。

---

## 4. 独自エラー型

`Box<dyn Error>` は手軽だが、呼び出し元が「**どの種類の失敗か**」で分岐しにくい。失敗の種類が決まっているなら `enum` で列挙する。README 課題 4（簡易電卓）を例にする。

### 1. 失敗の種類を `enum` で列挙する

```rust
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum CalcError {
    Parse(ParseIntError), // 元のエラーを包む
    UnknownOp(String),
    DivByZero,
    Overflow,
    WrongArity(usize),
}
```

### 2. `Display` を実装する（ユーザー向けの文言）

```rust
use std::fmt;

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::Parse(_) => write!(f, "数値として読めない"),
            CalcError::UnknownOp(op) => write!(f, "未知の演算子: {op}"),
            CalcError::DivByZero => write!(f, "0 で割ろうとした"),
            CalcError::Overflow => write!(f, "計算結果が i64 の範囲を超えた"),
            CalcError::WrongArity(n) => write!(f, "「数 演算子 数」の 3 要素が必要（{n} 要素だった）"),
        }
    }
}
```

`Debug`（`{:?}`）は開発者向け、`Display`（`{}`）はユーザー向け。`Display` には**原因（包んだエラー）の文言を重ねて書かない**。原因は次の `source()` でたどれるようにする（重ねると、原因まで表示するツールで同じ文言が 2 回出る）。

### 3. `std::error::Error` を実装する

```rust
use std::error::Error;

impl Error for CalcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CalcError::Parse(e) => Some(e), // 原因となったエラー
            _ => None,
        }
    }
}
```

`Error` trait は `Debug + Display` を前提にしており、メソッドは全部デフォルト実装があるので、原因を持たないなら `impl Error for CalcError {}` だけでよい。`Error` を実装すると `Box<dyn Error>` に入れられるようになる。

### 4. `From` を実装して `?` で自動変換させる

```rust
impl From<ParseIntError> for CalcError {
    fn from(e: ParseIntError) -> Self {
        CalcError::Parse(e)
    }
}

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
        "/" => a.checked_div(b).ok_or(CalcError::Overflow), // i64::MIN / -1
        _ => Err(CalcError::UnknownOp(op.to_string())),
    }
}
```

`From` を消すとコンパイルエラーになるが、書き方によってエラー番号が変わる（実際に試すと面白い）:

- `a.parse::<i64>()?` と型を明示した場合 → E0277「`?` couldn't convert the error to `CalcError`」。素直なエラー
- 上のように `let a: i64 = a.parse()?;` と書いた場合 → E0271「type mismatch resolving `<i64 as FromStr>::Err == CalcError`」。`From` 変換の手がかりが無いので、コンパイラが「`parse` のエラー型がそのまま `CalcError` であるはず」と推論してしまい、`i64` のエラー型（`ParseIntError`）と食い違う

どちらも原因は同じで、直し方は `impl From<ParseIntError> for CalcError` を書くこと。`From` の代わりに `.map_err(CalcError::Parse)?` と書いてもよい（列挙子はそれ自体が関数として使える）。

`checked_add` などは、オーバーフローすると `None` を返す。普通の `+` は debug ビルドでは panic、release ビルドでは黙って折り返す（wrap）。外部から来た数値を計算するなら `checked_*` で `Option` にし、`ok_or` でエラーに変える。

```text
"12 + 3" = 15
"7 / 0" → 0 で割ろうとした
"1 ^ 2" → 未知の演算子: ^
"x * 2" → 数値として読めない
"1 +" → 「数 演算子 数」の 3 要素が必要（2 要素だった）
"9223372036854775807 + 1" → 計算結果が i64 の範囲を超えた
```

### 原因の連鎖をたどる

```rust
fn print_chain(e: &dyn Error) {
    println!("error: {e}");
    let mut cur = e.source();
    while let Some(cause) = cur {
        println!("  caused by: {cause}");
        cur = cause.source();
    }
}
// error: 数値として読めない
//   caused by: invalid digit found in string
```

### `Box<dyn Error>` にすると型が消える

```rust
fn run_line(expr: &str) -> Result<i64, Box<dyn Error>> {
    let v = calc(expr)?; // CalcError → Box<dyn Error>
    Ok(v)
}

if let Err(e) = run_line("7 / 0") {
    // 具体的な型を取り出すには downcast_ref（失敗すると None）
    if let Some(CalcError::DivByZero) = e.downcast_ref::<CalcError>() { /* ... */ }
}
```

呼び出し元が種類で分岐する必要があるなら独自 `enum` のまま返し、表示して終わるだけなら `Box<dyn Error>` で十分。

### 定番 crate: `thiserror` と `anyhow`（紹介のみ）

上の `Display` / `Error` / `From` の手書きは定型的なので、実務では crate で省略することが多い。仕組みを理解するまでは手で書くこと。

```rust
// thiserror: 独自エラー型の定型コードを derive で生成する（ライブラリ向け）
#[derive(Debug, thiserror::Error)]
enum CalcError {
    #[error("数値として読めない")]
    Parse(#[from] ParseIntError), // From と source() を自動生成
    #[error("0 で割ろうとした")]
    DivByZero,
}

// anyhow: 何でも入るエラー型 + 文脈の付与（アプリ向け。Box<dyn Error> の強化版）
use anyhow::Context;
fn load(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("{path} を読めない"))
}
```

使うときはコンテナ内で `cargo add thiserror -p <crate名>` のように追加する。

**JS との違い**: JS でも `class CalcError extends Error` で独自エラーは作れるが、`catch (e)` の `e` は型が `unknown` で、何が飛んでくるかは保証されない。Rust の `Result<i64, CalcError>` は「失敗するならこの 5 種類のどれか」を型で約束し、呼び出し側の `match` で網羅性まで検査される。

---

## 5. `panic!` と `unwrap` / `expect`

`panic!` はスレッドを停止させる（スタックを巻き戻しながら各値を drop する）。main スレッドで起きればプログラムが終了する。

```rust
panic!("ここには来ないはず");
let n: i32 = "abc".parse().unwrap();
// called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
```

`unwrap()` は「`Some` / `Ok` なら中身、そうでなければ panic」。`expect(msg)` は panic 時のメッセージを指定できる版。

### 使ってよい場面

| 場面 | 理由 |
|---|---|
| テストコード | 失敗したらテストが落ちればよい |
| 論理的に失敗し得ないとき | 例: 直前に空でないことを確認済み。`expect` に**なぜ失敗しないか**を書く |
| 不変条件が破れたとき | 続行するとデータが壊れる。早く止めた方が安全 |
| プロトタイプ・使い捨てスクリプト | ただし後で `?` に置き換える前提 |

```rust
let v = [1, 2, 3];
let first = v.first().expect("v は 3 要素の配列なので空ではない");
```

`expect` のメッセージは「何が起きたか」ではなく「**なぜ起きないはずだったか**」を書くと、panic したときに前提のどこが崩れたか分かる。

それ以外（ユーザー入力・ファイル・ネットワークなど外から来るもの）は `Result` で返す。

### デバッグ

```bash
RUST_BACKTRACE=1 cargo run -p errors   # panic した位置までの呼び出し履歴を表示
```

テストでは `#[should_panic(expected = "...")]` で「panic すること」を検査できる。また、テスト関数自体も `Result` を返して `?` を使える。

```rust
#[test]
#[should_panic(expected = "InvalidDigit")]
fn unwrap_panics() {
    let _n: i32 = "abc".parse().unwrap();
}

#[test]
fn question_in_test() -> Result<(), CalcError> {
    assert_eq!(calc("2 * 21")?, 42);
    Ok(())
}
```

**JS との違い**: JS ではバグ（`undefined` のプロパティ参照）も回復可能な失敗（ネットワークエラー）も同じ `throw` で、`catch` で区別なく捕まる。Rust は `panic!`（バグ・回復しない）と `Result`（想定内の失敗・回復する）を**仕組みごと分けている**。panic を `Result` のように捕まえて続行する書き方（`catch_unwind`）は存在するが、通常のエラー処理には使わない。

---

## 6. よくあるコンパイルエラーと直し方

エラー番号の詳細は `rustc --explain E0277` で読める。

| エラー | メッセージ（要約） | 原因 | 典型的な直し方 |
|---|---|---|---|
| E0308 | mismatched types: expected `&str`, found `Option<&str>` | `Option` / `Result` を中身の型として使った | `match` / `if let` / `?` / `unwrap_or` で取り出す |
| E0277 | the `?` operator can only be used in a function that returns `Result` or `Option` | 戻り値が `()` の関数で `?` を使った | 戻り値を `Result<(), E>` にする（`main` も可）／`match` で処理する |
| E0277 | the `?` operator can only be used on `Option`s, not `Result`s, in a function that returns `Option` | `Option` を返す関数で `Result` に `?` を使った（逆も同様） | `.ok()?` で `Option` に寄せる／`.ok_or(e)?` で `Result` に寄せる |
| E0277 | `?` couldn't convert the error to `MyError` | `From<元のエラー>` が `MyError` に実装されていない | `impl From<...> for MyError` を書く／`.map_err(MyError::Variant)?` |
| E0271 | type mismatch resolving `<i64 as FromStr>::Err == MyError` | 上と同じ原因で、型推論の経路が違うと出る（`let a: i64 = s.parse()?;` の形） | 上と同じ |
| E0004 | non-exhaustive patterns: `None` not covered | `match` で `None` / `Err` の腕を書き忘れた | 腕を足す／本当に不要なら `if let` にする |
| 警告 | unused `Result` that must be used | `Result` を返す関数の戻り値を無視した | `?` で伝播する／`let _ = ...;` で明示的に捨てる |

読み方のコツ: E0277 は「trait が実装されていない」という汎用のエラー番号なので、番号ではなく**メッセージの本文**を読む。`?` 関連なら「どの型からどの型へ変換しようとして失敗したか」が書いてある。

---

## 7. 設計の指針

- **「無い」と「失敗」を区別する**: 見つからないのが普通なら `Option`、理由を伝えたいなら `Result`。迷ったら呼び出し元が「なぜ？」を知りたいかで決める
- **ライブラリ的なコードは具体的なエラー型、アプリの上の方は `Box<dyn Error>`**: 呼び出し元が種類で分岐するなら `enum`。`main` に近い層は表示して終わるだけなので `Box<dyn Error>`（あるいは `anyhow`）で十分
- **エラーは 1 か所で処理する**: 途中の層では `?` で上に渡すだけにし、ログ出力やユーザーへの表示は最上位でまとめて行う。各層で `println!` してから `Err` を返すと、同じエラーが何度も表示される
- **外から来るものは `Result`、内部の約束違反は `panic!`**: ユーザー入力・ファイル・数値の変換は必ず失敗し得る。`unwrap` は「ここで失敗したらバグ」と言えるときだけ
- **不変条件はコンストラクタで守る**: 「正しい状態の値しか作れない」ようにすれば、使う側でのチェックが不要になる

最後の点はステージ2（仕訳・試算表アプリ）の中心になる。貸借が一致しない仕訳を**そもそも作れない**ようにするスケッチ:

```rust
#[derive(Debug, Clone, Copy)]
enum Side { Debit, Credit } // 借方 / 貸方

#[derive(Debug, PartialEq)]
enum JournalError {
    Empty,
    ZeroAmount,
    Unbalanced { debit: u64, credit: u64 },
}

/// 貸借が一致した仕訳。new を通らないと作れないので、存在する時点で一致が保証される
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
        let (debit, credit) = lines.iter().fold((0, 0), |(d, c), &(side, amount)| match side {
            Side::Debit => (d + amount, c),
            Side::Credit => (d, c + amount),
        });
        if debit != credit {
            return Err(JournalError::Unbalanced { debit, credit });
        }
        Ok(Journal { lines })
    }
}
```

フィールド `lines` を非公開にしておけば（モジュールを分けたとき `pub` を付けない）、外部から `Journal { lines }` で直接作ることもできない。試算表の集計側は「`Journal` があるなら貸借は一致している」と信じてよくなる。

---

## 8. 理解度チェック

答えを見ずに、自分の言葉で説明できるか確かめる（マトリクスの S4 を埋める前に）。

- [ ] `Option<T>` があると、なぜ null 参照のバグが起きないのか
- [ ] 配列の `v.get(i)` と `v[i]` の違い。それぞれどんなときに使うか
- [ ] `map` と `and_then` の違い。`map` に `Option` を返すクロージャを渡すとどうなるか
- [ ] `unwrap_or(expensive())` と `unwrap_or_else(|| expensive())` の違い
- [ ] `x?` を `match` で書き直す。`From::from` が挟まるのはなぜか
- [ ] `fn main() { "1".parse::<i32>()?; }` がエラーになる理由と、2 通りの直し方
- [ ] `Box<dyn Error>` と独自エラー `enum` の使い分け
- [ ] 独自エラー型で `Display` に原因の文言を含めない理由
- [ ] `expect` のメッセージには何を書くべきか
- [ ] `panic!` と `Err` を返すことの使い分けを、具体例 2 つずつで説明する
- [ ] 仕訳のコンストラクタが `Result` を返すと、アプリの他の部分で何が楽になるか
- [ ] JS の `try-catch` と比べて、Rust のエラー処理が「型に現れる」とはどういうことか
