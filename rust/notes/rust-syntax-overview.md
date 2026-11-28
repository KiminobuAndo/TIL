# Rust 基本文法と Rust 固有の特徴まとめ

> `rust-3layer-matrix.md` を埋めるときの参照用リファレンス。
> 見出しの ID（L1〜L7 / P1〜P3 / S1〜S8）はマトリクスの行と対応している。
> マトリクスには**このファイルを写さず**、自分の言葉で書くこと。

---

## 目次

1. [基本文法（言語普遍層）](#1-基本文法言語普遍層)
2. [パラダイム層](#2-パラダイム層)
3. [Rust 固有の特徴（他言語にないもの）](#3-rust-固有の特徴他言語にないもの)
4. [他言語との比較早見表](#4-他言語との比較早見表)

---

## 1. 基本文法（言語普遍層）

### L0 字句

- コメント: `//` 行コメント、`/* */` ブロック、`///` ドキュメントコメント（`cargo doc` で HTML になる）
- 命名規則はコンパイラが警告する: 変数・関数は `snake_case`、型・trait は `CamelCase`、定数は `SCREAMING_SNAKE_CASE`
- 末尾の `!` はマクロ呼び出しの印（`println!`）。→ S8

### L1 値と型

```rust
let a: i32 = -42;        // 符号付き整数: i8 / i16 / i32 / i64 / i128 / isize
let b: u64 = 42;         // 符号なし整数: u8 〜 u128 / usize
let i: usize = 0;        // 配列の添字や長さはポインタ幅の usize
let x: f64 = 3.14;       // 浮動小数: f32 / f64
let t: bool = true;
let c: char = 'あ';      // char は Unicode スカラー値 1 つ = 4 バイト
let big = 1_000_000;     // _ で桁区切りできる（型を書かなければ i32）

let s1: &str = "hello";              // 文字列スライス（借用。リテラルはバイナリに埋め込まれる）
let s2: String = String::from("hi"); // ヒープ上の可変長文字列（所有する）

let tup: (i32, f64, char) = (1, 2.0, 'z');
let (p, q, r) = tup;     // 分割代入
let arr: [i32; 3] = [1, 2, 3];       // 固定長配列。長さも型の一部
```

- **暗黙の型変換は一切ない**。`i32 + i64` はコンパイルエラーで、`x as i64` のように明示する
- 整数オーバーフローは debug ビルドで panic、release ビルドでは折り返す（明示したいときは `wrapping_add` / `checked_add`）
- 文字列は UTF-8。`s[0]` のような添字アクセスはできない（1 文字が何バイトか分からないため）。`s.chars()` を使う

### L2 式（Expression）

Rust は**ほぼすべてが式**で、値を返す。

```rust
let n = 7;
let kind = if n % 2 == 0 { "even" } else { "odd" };  // if は式（三項演算子は無い）

let y = {
    let a = 3;
    a * 2        // ← セミコロンが無い最後の式がブロックの値になる
};               // y == 6

let z = {
    let a = 3;
    a * 2;       // ← セミコロンを付けると文になり、ブロックの値は () （unit 型）
};               // z == ()
```

**末尾のセミコロンの有無で意味が変わる**のが最初のつまずきポイント。

### L3 文と制御フロー

```rust
// loop: 無限ループ。break に値を渡すと loop 式の値になる
let mut count = 0;
let result = loop {
    count += 1;
    if count == 10 {
        break count * 2;   // result == 20
    }
};

// while
while count > 0 {
    count -= 1;
}

// for: イテレータを回す。範囲は 1..10（10 を含まない）/ 1..=10（含む）
for i in 1..=3 {
    println!("{i}");
}
for (idx, v) in ["a", "b"].iter().enumerate() {
    println!("{idx}: {v}");
}

// ラベル付き break で外側のループを抜ける
'outer: for x in 0..5 {
    for y in 0..5 {
        if x * y == 6 {
            break 'outer;
        }
    }
}
```

- C 形式の `for (i = 0; i < n; i++)` は無い
- 条件式に括弧は不要、本体の `{}` は必須。条件は `bool` のみ（`if 1 {}` はエラー。truthy/falsy が無い）

### L4 束縛とスコープ

```rust
let x = 5;
// x = 6;          // エラー: 変数はデフォルトで不変
let mut y = 5;
y = 6;             // mut を付けたものだけ再代入できる

let x = x + 1;     // shadowing: 同じ名前で新しい変数を作る（型を変えてもよい）
let spaces = "   ";
let spaces = spaces.len();   // &str → usize に変えて同名で使える

const MAX_POINTS: u32 = 100_000;   // コンパイル時定数。型注釈必須
static GREETING: &str = "hello";   // プログラム全体で 1 つのアドレスを持つ値
```

- `mut` は「この束縛を通して値を変更できるか」の宣言。JS の `const` と違い、`let` だと**中身の変更も**禁止される
- スコープ終端で値は破棄（drop）される。→ S1

### L5 関数と抽象化

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b            // 最後の式が戻り値（return は早期リターン用）
}

fn print_only(msg: &str) {   // 戻り値型を省略すると ()
    println!("{msg}");
}
```

- 引数と戻り値の型注釈は**必須**（関数内部は推論される）
- デフォルト引数・可変長引数・オーバーロードは無い（ビルダーパターンや trait で代替）

**クロージャ**:

```rust
let factor = 3;
let mul = |x: i32| x * factor;        // 環境の factor を借用してキャプチャ
println!("{}", mul(2));               // 6

let mut total = 0;
let mut add_to = |x| total += x;      // 環境を変更するので FnMut
add_to(5);

let name = String::from("rust");
let consume = move || name;           // move で所有権ごとキャプチャ → FnOnce
let owned = consume();
```

| trait | キャプチャの仕方 | 呼べる回数 |
|---|---|---|
| `Fn` | 不変借用 `&` | 何度でも |
| `FnMut` | 可変借用 `&mut` | 何度でも（ただし同時に 1 つ） |
| `FnOnce` | 所有権を奪う | 1 回だけ |

### L6 データ構造

```rust
// 構造体
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {          // 関連関数（コンストラクタの慣習名は new）
        Self { x, y }
    }
    fn dist(&self) -> f64 {                   // メソッド: 第1引数が self
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn shift(&mut self, dx: f64) {            // 自身を変更するメソッドは &mut self
        self.x += dx;
    }
}

let mut p = Point::new(3.0, 4.0);
p.shift(1.0);
println!("{:?} {}", p, p.dist());   // {:?} は Debug 表示

// タプル構造体 / ユニット構造体
struct Meters(f64);
struct Marker;

// 列挙型: 各バリアントがデータを持てる（代数的データ型）
enum Shape {
    Circle { r: f64 },
    Rect(f64, f64),
    Empty,
}

// コレクション
let mut v: Vec<i32> = vec![1, 2, 3];
v.push(4);
let slice: &[i32] = &v[1..3];          // スライス = 連続領域への参照 [2, 3]

use std::collections::HashMap;
let mut counts: HashMap<&str, i32> = HashMap::new();
*counts.entry("apple").or_insert(0) += 1;
```

- クラスは無い。**データ（`struct`）と振る舞い（`impl`）を分けて書く**
- `v[10]` のように範囲外を添字アクセスすると panic。安全に取るなら `v.get(10)`（`Option` が返る）

### L7 名前空間・モジュール

```rust
mod geometry {                 // モジュール（ファイル分割なら geometry.rs）
    pub struct Square(pub f64);        // pub を付けないと外から見えない（デフォルト非公開）
    pub fn area(s: &Square) -> f64 {
        s.0 * s.0
    }
}

use geometry::{area, Square};  // パスを取り込む
use std::collections::HashMap;

fn main() {
    println!("{}", area(&Square(2.0)));
}
```

- **crate**: コンパイル単位（`Cargo.toml` 1 つ = package。中に bin / lib crate）
- パスの起点: `crate::`（自 crate ルート）、`self::`、`super::`（親モジュール）
- 外部 crate は `cargo add <name>` で `Cargo.toml` に追加して `use` する

---

## 2. パラダイム層

### P1 オブジェクト指向 — 継承ではなく trait による合成

```rust
trait Speak {
    fn name(&self) -> String;
    fn speak(&self) -> String {               // デフォルト実装を持てる
        format!("{} がしゃべった", self.name())
    }
}

struct Dog;
impl Speak for Dog {
    fn name(&self) -> String { "犬".into() }
}

// 動的ディスパッチ（実行時に決まる）: dyn Trait
let animals: Vec<Box<dyn Speak>> = vec![Box::new(Dog)];
for a in &animals {
    println!("{}", a.speak());
}
```

- **クラス継承は無い**。カプセル化は `pub` / モジュール、ポリモーフィズムは trait で行う
- 多態は 2 種類: ジェネリクス（静的ディスパッチ、コンパイル時に型ごとに展開）と `dyn Trait`（動的ディスパッチ、vtable 経由）

### P2 関数型 — `Iterator` の遅延評価

```rust
let nums = vec![1, 2, 3, 4, 5, 6];

let sum_of_even_squares: i32 = nums
    .iter()                    // イテレータを作る（この時点では何も計算しない）
    .filter(|&&n| n % 2 == 0)
    .map(|&n| n * n)
    .sum();                    // 消費メソッドを呼んだ時点で初めて計算される

let doubled: Vec<i32> = nums.iter().map(|n| n * 2).collect();
let total = nums.iter().fold(0, |acc, n| acc + n);
```

- `map` / `filter` は**遅延**。`collect` / `sum` / `for` などで消費するまで何も起きない（JS の配列メソッドは即時評価で中間配列を作る）
- `iter()`（`&T` を返す）/ `iter_mut()`（`&mut T`）/ `into_iter()`（`T`、所有権を消費）の 3 種類を使い分ける
- コンパイル後は手書きループと同等の速度になる（ゼロコスト抽象化）

### P3 非同期・並行

```rust
// スレッド: 所有権規則のおかげでデータ競合はコンパイルエラーになる
use std::thread;
let data = vec![1, 2, 3];
let handle = thread::spawn(move || data.iter().sum::<i32>());
println!("{}", handle.join().unwrap());

// async/await: 構文は言語にあるが、実行するランタイム（tokio など）は外部 crate
// async fn fetch() -> String { ... }
// #[tokio::main] async fn main() { let s = fetch().await; }
```

- `Send`（他スレッドへ移動できる）/ `Sync`（他スレッドから共有参照できる）という marker trait で、スレッド安全性を**型で**検査する（"fearless concurrency"）
- `async fn` は呼んだだけでは実行されない（`Future` を返すだけ。`.await` されて初めて進む）

---

## 3. Rust 固有の特徴（他言語にないもの）

Rust の中心思想は「**GC 無しで、メモリ安全性とデータ競合の不在をコンパイル時に保証する**」こと。S1〜S3 はそのための仕組みで、他の主要言語にはほぼ存在しない。

> S1〜S3 の詳しい解説・エラー対処は [`rust-ownership-borrowing.md`](rust-ownership-borrowing.md) を参照。

### S1 所有権（Ownership）

**ルール**:

1. 各値には**所有者（owner）がちょうど 1 つ**
2. 所有者がスコープを抜けると値は**自動で破棄（drop）**される
3. 代入や関数への引数渡しで、所有権は**ムーブ（移動）**する

```rust
let s1 = String::from("hello");
let s2 = s1;                 // 所有権が s1 → s2 にムーブ。s1 はもう使えない
// println!("{s1}");         // error[E0382]: borrow of moved value: `s1`

let s3 = s2.clone();         // 明示的なディープコピー。s2 も s3 も使える

let a = 5;
let b = a;                   // i32 は Copy 型なのでムーブではなくコピー。a も使える

fn take(s: String) {}        // 引数に渡すとムーブ
take(s3);
// take(s3);                 // エラー: 2 回目はもう所有していない
```

| | `Copy` | `Clone` |
|---|---|---|
| 発生 | 代入・引数渡しで**暗黙に** | `.clone()` で**明示的に** |
| 対象 | スタックだけで完結する型（整数、`bool`、`char`、`&T`、それらのタプル） | ヒープを持つ型も可（`String`、`Vec`） |
| コスト | ビットコピーのみ（安い） | 任意（高いこともある） |

**なぜあるか**: GC 無しで「いつ解放するか」を静的に決めるため。二重解放・解放後使用（use-after-free）・メモリリークの大部分がコンパイル時に排除される。C++ の RAII を言語規則として強制したもの。

### S2 借用と参照（`&` / `&mut`）

所有権を渡さずに値を使わせるのが**借用**。

```rust
fn len(s: &str) -> usize {         // 不変借用: 読むだけ（&String は &str に自動変換される）
    s.len()
}
fn push_world(s: &mut String) {    // 可変借用: 変更できる
    s.push_str(" world");
}

let mut s = String::from("hello");
let n = len(&s);
push_world(&mut s);
```

**借用規則**（同時に成り立つのはどちらか一方）:

- 不変参照 `&T` は**いくつでも**同時に存在できる
- 可変参照 `&mut T` は**ただ 1 つだけ**で、その間は不変参照も持てない

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];           // 不変借用
// v.push(4);                // error[E0502]: cannot borrow `v` as mutable
                             //   because it is also borrowed as immutable
println!("{first}");         // first の最後の使用（ここで借用が終わる）
v.push(4);                   // ここなら OK（NLL: 借用は最後に使った位置で終わる）
```

**なぜあるか**: 「共有」と「変更」を同時に許さない（aliasing XOR mutability）ことで、イテレータ無効化やデータ競合をコンパイル時に防ぐ。上の例は `push` で Vec が再確保されると `first` がダングリングポインタになる、という C++ で実際に起きるバグ。

### S3 ライフタイム（`'a`）

参照が**指している先より長生きしない**ことをコンパイラが検証する。多くは推論されるが、関数が複数の参照を受けて参照を返す場合などは注釈が要る。

```rust
// 戻り値の参照は x と y の短い方と同じだけ生きる、という宣言
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// ダングリング参照はコンパイルエラー
// fn dangle() -> &String {
//     let s = String::from("hi");
//     &s                     // error[E0106]: missing lifetime specifier
// }                          // s はここで drop されるので参照を返せない

// 参照を持つ構造体にも注釈が要る
struct Excerpt<'a> {
    part: &'a str,
}
```

- ライフタイム注釈は**寿命を変えない**。複数の参照の関係をコンパイラに説明するだけ
- `'static` はプログラム終了まで有効（文字列リテラルなど）
- 初学のうちは「参照を返すより所有した値（`String`）を返す」で回避してよい

### S4 `Option` / `Result` — null と例外が無い

```rust
// null の代わり: 値があるかもしれない
fn find(v: &[i32], target: i32) -> Option<usize> {
    v.iter().position(|&x| x == target)
}

match find(&[1, 2, 3], 2) {
    Some(i) => println!("index {i}"),
    None => println!("not found"),
}
let i = find(&[1, 2, 3], 9).unwrap_or(0);   // 無ければデフォルト値

// 例外の代わり: 成功か失敗か
use std::num::ParseIntError;

fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.trim().parse()?;   // ? : Err なら即座にその Err を return する
    Ok(n * 2)
}

match parse_and_double("21") {
    Ok(v) => println!("{v}"),
    Err(e) => println!("error: {e}"),
}
```

- `Option<T>` と `T` は別の型なので、**`None` かもしれない値をうっかり使うとコンパイルエラー**になる（「10 億ドルの誤り」の null 参照が起きない）
- エラーが**戻り値の型に現れる**ので、失敗しうる関数かどうかがシグネチャで分かる
- `?` 演算子はエラーを呼び出し元へ伝播する（`From` trait でエラー型の変換も自動で行う）
- `unwrap()` / `expect("msg")` は `None` / `Err` で panic する。学習中・テスト以外では避ける
- 回復不能なバグは `panic!`（スタックを巻き戻してスレッドを終了）。回復可能なエラーは `Result`、と使い分ける

> S4 の詳しい解説（コンビネータ、`?` の仕組み、独自エラー型、`panic!` との使い分け）は [`rust-error-handling.md`](rust-error-handling.md) を参照。

### S5 trait とジェネリクス

```rust
use std::fmt::Display;

// trait 境界: T は PartialOrd（比較可能）と Copy（コピー可能）を実装している型に限る
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut max = list[0];
    for &item in list {
        if item > max { max = item; }
    }
    max
}

// where 句で書いても同じ
fn show<T>(x: T) -> String
where
    T: Display,
{
    format!("<{x}>")
}

// impl Trait: 引数・戻り値で「この trait を実装した何か」
fn evens() -> impl Iterator<Item = u32> {
    (0..10).filter(|n| n % 2 == 0)
}

// 既存の型に自分の trait を後付けできる
trait Double { fn double(&self) -> Self; }
impl Double for i32 {
    fn double(&self) -> Self { self * 2 }
}
println!("{}", 21_i32.double());   // 42（型未確定の 21 だとメソッド解決できず E0689）

// derive: よく使う trait の実装を自動生成
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Id(u32);
```

- ジェネリクスは**単相化（monomorphization）**される: `largest::<i32>` と `largest::<f64>` が別々の機械語として生成されるので実行時コストがゼロ
- **orphan rule**: 「外部の trait を外部の型に実装する」ことはできない（trait か型のどちらかが自 crate のものである必要がある）
- 演算子オーバーロードも trait（`std::ops::Add` を実装すると `+` が使える）

> S5 の詳しい解説（デフォルト実装、trait 境界、`dyn Trait` との使い分け、関連型、標準 trait、orphan rule）は [`rust-traits-generics.md`](rust-traits-generics.md) を参照。

### S6 パターンマッチ（`match`）— 網羅性検査

```rust
enum Coin { Penny, Nickel, Dime, Quarter }

fn value(c: Coin) -> u32 {
    match c {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }   // どれか 1 つでも書き漏らすと error[E0004]: non-exhaustive patterns
}

let n = 7;
let msg = match n {
    0 => "zero",
    1 | 2 => "small",              // OR パターン
    3..=9 => "single digit",       // 範囲パターン
    x if x < 0 => "negative",      // ガード
    _ => "large",                  // ワイルドカード
};

// 構造の分解
let p = (3, -3);
match p {
    (0, y) => println!("on y axis at {y}"),
    (x, y) if x == -y => println!("on diagonal"),
    _ => {}
}

// 1 パターンだけ見たいとき
let maybe: Option<i32> = Some(3);
if let Some(v) = maybe {
    println!("{v}");
}

// let-else: マッチしなければ早期 return（ガード節）
fn parse_pos(s: &str) -> Option<u32> {
    let Ok(n) = s.parse::<u32>() else {
        return None;
    };
    Some(n)
}
```

- **enum にバリアントを追加すると、対応漏れの `match` がすべてコンパイルエラーになる**。リファクタリングの安全網として強力
- `match` は式なので値を返せる（L2）

### S7 スマートポインタ — `Box` / `Rc` / `RefCell`

「所有の形」を型で選ぶ。

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Box<T>: ヒープに置き、単独所有。再帰的な型に必須（サイズを確定させるため）
enum List {
    Cons(i32, Box<List>),
    Nil,
}

// Rc<T>: 参照カウントによる共有所有（シングルスレッド）。中身は不変
let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);                 // カウント +1（ディープコピーではない）
println!("{}", Rc::strong_count(&a));  // 2

// RefCell<T>: 借用規則を実行時に検査する（内部可変性）
let cell = RefCell::new(vec![1]);
cell.borrow_mut().push(2);             // 不変の束縛越しに変更できる
// 同時に borrow_mut を 2 回すると実行時に panic

// 組み合わせ: 共有しつつ変更したい
let shared = Rc::new(RefCell::new(0));
let other = Rc::clone(&shared);
*other.borrow_mut() += 10;
println!("{}", shared.borrow());       // 10
```

| 型 | 所有 | 変更 | 検査 | スレッド版 |
|---|---|---|---|---|
| `Box<T>` | 単独 | 可（`mut` なら） | コンパイル時 | そのまま使える |
| `Rc<T>` | 共有 | 不可 | コンパイル時 | `Arc<T>` |
| `RefCell<T>` | 単独 | 可（`&` 越しでも） | **実行時** | `Mutex<T>` / `RwLock<T>` |

- `Rc` 同士で循環参照を作るとリークする → 片方を `Weak<T>` にする

### S8 マクロ

```rust
println!("{} + {} = {}", 1, 2, 1 + 2);  // 可変長引数・書式チェックはマクロでしかできない
let v = vec![1, 2, 3];
assert_eq!(v.len(), 3);
let s = format!("{:>5}", 42);           // 右寄せ 5 桁

// 宣言的マクロ（macro_rules!）
macro_rules! square {
    ($x:expr) => { $x * $x };
}
println!("{}", square!(4));             // 16
```

- **コンパイル時に構文木を生成**する（C のプリプロセッサのような文字列置換ではなく、衛生的）
- 書式文字列の引数の数・型の不一致はコンパイルエラーになる
- `#[derive(...)]` や `#[tokio::main]` は**手続き的マクロ**（Rust コードで Rust コードを生成する）

### その他の Rust らしさ

| 特徴 | 内容 |
|---|---|
| デフォルト不変 | `let` は不変、変更したいものだけ `mut`。参照も `&` が不変で `&mut` が例外 |
| 暗黙の型変換なし | 数値の型が違えば `as` か `From`/`Into` で明示する |
| `unsafe` | 生ポインタ参照や FFI など、コンパイラが保証できない操作を囲って**範囲を限定**する。安全な API の内側に閉じ込めるのが作法 |
| ゼロコスト抽象化 | イテレータ・ジェネリクス・`Option<&T>` などが手書きと同等の機械語になる |
| GC もランタイムも無い | 起動が速く、メモリ使用量が予測できる。C ライブラリとして組み込める |
| Cargo | ビルド・依存管理・テスト・ドキュメント・フォーマッタ（`cargo fmt`）・リンタ（`cargo clippy`）が標準で一体化 |
| テストが言語に組み込み | `#[test]` を付けた関数を `cargo test` で実行。同じファイルに `#[cfg(test)] mod tests` を書くのが慣習 |
| コンパイラのエラーメッセージ | 修正案まで示してくれる。`rustc --explain E0382` で詳細解説が読める |

---

## 4. 他言語との比較早見表

| 概念 | Rust | JavaScript | Java | Go | C |
|---|---|---|---|---|---|
| 型付け | 静的・強い推論 | 動的 | 静的 | 静的・簡易推論 | 静的・弱い |
| メモリ管理 | **所有権（コンパイル時）** | GC | GC | GC | 手動 `malloc`/`free` |
| null | **無い**（`Option<T>`） | `null` / `undefined` | `null`（`Optional` あり） | `nil` | `NULL` |
| エラー処理 | `Result<T, E>` + `?` | 例外 | 検査例外 + 例外 | 戻り値 `(v, err)` | 戻り値・`errno` |
| 変数の既定 | **不変** | 可変（`const` で再代入禁止） | 可変（`final`） | 可変 | 可変（`const`） |
| 継承 | **無い**（trait） | プロトタイプ | クラス継承 | 無い（埋め込み + interface） | 無い |
| インターフェース | trait（明示的に `impl`） | 無い（ダックタイピング） | `interface`（明示的） | interface（暗黙的に満たす） | 無い |
| ジェネリクス | あり（単相化） | 不要 | あり（型消去） | あり | 無い（マクロ / `void*`） |
| パターンマッチ | `match`（網羅性検査） | 無い（`switch`） | `switch` パターン（新しめ） | `switch`（網羅性なし） | `switch` |
| 式指向 | `if` / `match` / ブロックが式 | 文 | 文（`switch` 式は一部） | 文 | 文 |
| 並行処理 | スレッド + `async`、**データ競合はコンパイルエラー** | シングルスレッド + イベントループ | スレッド | goroutine + channel | スレッド（ライブラリ） |
| ダングリング参照 | **コンパイルエラー** | 起きない（GC） | 起きない（GC） | 起きない（GC） | 未定義動作 |
| ビルド/パッケージ | Cargo（標準） | npm 等 | Maven / Gradle | go コマンド（標準） | make 等 |

**一言でまとめると**: Rust は「C と同じ速度・制御性」と「GC 言語と同じメモリ安全性」を、**所有権・借用・ライフタイムという型システムの規則**で両立させた言語。その代償として、他言語ならコンパイルが通るコードの一部が通らない。コンパイラとの対話（エラーを読んで直す）がそのまま学習になる。
