# trait とジェネリクス（S5 深掘り）

> `rust-syntax-overview.md` の S5（と P1 の `dyn Trait`）を詳しく掘り下げたリファレンス。
> サンプルコードは `exercises/traits/src/main.rs` に関数単位でまとめてあり、`cargo run -p traits` でそのまま動く（テストは `cargo test -p traits`）。
> エラーになる例はコメントアウトしてあるので、外してエラーメッセージを実際に読むこと。
> マトリクス（`rust-3layer-matrix.md`）の S5 には**このファイルを写さず**、自分の言葉で書くこと。

---

## 目次

0. [前提: trait は「振る舞いの約束」](#0-前提-trait-は振る舞いの約束)
1. [trait の定義と実装](#1-trait-の定義と実装)
2. [ジェネリクス](#2-ジェネリクス)
3. [trait 境界](#3-trait-境界)
4. [静的ディスパッチと動的ディスパッチ（`dyn Trait`）](#4-静的ディスパッチと動的ディスパッチdyn-trait)
5. [関連型と標準 trait](#5-関連型と標準-trait)
6. [orphan rule と newtype](#6-orphan-rule-と-newtype)
7. [よくあるコンパイルエラーと直し方](#7-よくあるコンパイルエラーと直し方)
8. [設計の指針](#8-設計の指針)
9. [理解度チェック](#9-理解度チェック)

---

## 0. 前提: trait は「振る舞いの約束」

trait は「この型はこういうメソッドを持っている」という**約束（インターフェース）**。ジェネリクスは「型を引数にとる」仕組みで、trait と組み合わせて「**この約束を満たす任意の型**」を受け取る関数や型を書く。

| 観点 | Rust の trait | TypeScript の interface | JS（素） |
|---|---|---|---|
| 型が約束を満たす条件 | `impl Trait for Type` と**明示的に書く**（名目的） | 形が合えばよい（構造的） | 実行時に呼んでみて動けばよい（ダックタイピング） |
| 検査のタイミング | コンパイル時 | コンパイル時（実行時には消える） | 実行時（`TypeError: x.foo is not a function`） |
| 既存の型への後付け | できる（自分の trait なら `i32` にも実装できる） | 宣言のマージで一応可 | `prototype` を書き換える（推奨されない） |
| デフォルト実装 | 持てる | 持てない（abstract class なら可） | — |
| 静的メソッド・定数の約束 | できる（関連関数・関連定数） | できない | — |
| 実行時のコスト | ジェネリクスなら 0（単相化）、`dyn` なら vtable 1 回分 | — | プロパティ探索 |

```rust
trait Shape {
    fn area(&self) -> f64;
}

// 形が同じでも、impl を書かない限り Shape ではない
struct Rect { w: f64, h: f64 }
impl Shape for Rect {
    fn area(&self) -> f64 { self.w * self.h }
}
```

**クラス継承は無い**。Rust でのポリモーフィズムはすべて trait で行う。「データ（`struct` / `enum`）」と「振る舞い（trait）」を別々に定義し、`impl` で結び付ける、という分離が基本の考え方。

---

## 1. trait の定義と実装

### 必須メソッドとデフォルト実装

```rust
trait Shape {
    fn area(&self) -> f64;            // 必須: 本体なし

    fn name(&self) -> String {         // デフォルト実装: 上書きしてもしなくてもよい
        "図形".to_string()
    }

    fn describe(&self) -> String {     // デフォルト実装から必須メソッドを呼べる
        format!("{}（面積 {:.2}）", self.name(), self.area())
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.r * self.r }
    fn name(&self) -> String { "円".to_string() }   // 上書き
}

impl Shape for Rect {
    fn area(&self) -> f64 { self.w * self.h }
    // name を書かない → "図形" が使われる
}
```

- 必須メソッドを 1 つ書くだけで、デフォルト実装のメソッドがまとめて使えるようになる。標準ライブラリの `Iterator` は `next` を書くだけで `map` / `filter` / `sum` など 70 以上のメソッドが付く（→ 5 章）
- 必須メソッドを書き忘れると `E0046: not all trait items implemented`
- `r.area()` は `Shape::area(&r)` の糖衣構文。同名メソッドを持つ trait が 2 つあるときは後者で呼び分ける

### `self` の受け取り方

メソッドの第 1 引数は所有権のルール（S1〜S2）どおりに 3 通り。

| 書き方 | 意味 | 例 |
|---|---|---|
| `&self` | 読むだけ（借用） | `fn area(&self) -> f64` |
| `&mut self` | 書き換える（可変借用） | `fn next(&mut self) -> Option<Self::Item>` |
| `self` | 消費する（move） | `fn into_iter(self) -> Self::IntoIter` |

### 関連関数・関連定数・`Self`

`self` を取らない関数（関連関数）や定数も約束にできる。`Self` は「この trait を実装している型そのもの」を指す。

```rust
trait Unit {
    const SYMBOL: &'static str;      // 関連定数
    fn from_raw(v: i64) -> Self;      // 関連関数（コンストラクタの約束）
    fn raw(&self) -> i64;

    fn show(&self) -> String {
        format!("{}{}", self.raw(), Self::SYMBOL)
    }
}

impl Unit for Yen {
    const SYMBOL: &'static str = "円";
    fn from_raw(v: i64) -> Self { Yen(v) }
    fn raw(&self) -> i64 { self.0 }
}

// ジェネリクスの中で「型から値を作る」ことができる
fn doubled<U: Unit>(u: &U) -> U {
    U::from_raw(u.raw() * 2)
}

let y = Yen::from_raw(1500);
let m = <Meter as Unit>::from_raw(42);   // 完全修飾構文
```

**JS / TS との違い**: TS の interface はインスタンスの形しか約束できず、「この型には `fromRaw` という静的メソッドがある」とは書けない。Rust では `Default::default()` / `From::from()` / `FromStr::from_str()`（`parse` の中身）など、**コンストラクタを trait で抽象化する**のが日常的。

---

## 2. ジェネリクス

### ジェネリック関数

```rust
fn largest<T: PartialOrd + Copy>(list: &[T]) -> Option<T> {
    let mut iter = list.iter().copied();
    let mut max = iter.next()?;      // 空なら None（S4 の ? が Option にも使える）
    for x in iter {
        if x > max { max = x; }
    }
    Some(max)
}

largest(&[3, 7, 2]);         // T = i32
largest(&[1.5, -0.5]);       // T = f64
largest(&['a', 'z']);        // T = char
largest::<i32>(&[]);         // 引数から推論できないときはターボフィッシュ ::<>
```

- `<T>` は型の引数。呼び出し時に**引数の型から推論**される
- `T: PartialOrd + Copy` が **trait 境界**。「`T` は比較できて、コピーできる型に限る」。境界が無いと `x > max` が書けない（`E0369`）
- 境界は「関数の中で `T` に何をしてよいか」の宣言でもある。中身を見なくてもシグネチャだけで何ができる関数か分かる

`parse` や `collect` は**戻り値の型**で `T` が決まるジェネリクス。推論できないと `E0284` / `E0283`（type annotations needed）になる。

```rust
let n = "42".parse::<u8>();                          // ターボフィッシュで指定
let v: Vec<i32> = "1 2 3".split(' ').map(|s| s.parse().unwrap()).collect(); // 代入先で指定
```

### ジェネリックな `struct` / `enum` と `impl`

```rust
#[derive(Debug)]
struct Pair<T> { a: T, b: T }

impl<T> Pair<T> {                    // どんな T の Pair にも
    fn new(a: T, b: T) -> Self { Self { a, b } }
    fn swap(self) -> Self { Self { a: self.b, b: self.a } }
}

impl Pair<f64> {                     // Pair<f64> のときだけ
    fn mean(&self) -> f64 { (self.a + self.b) / 2.0 }
}

Pair::new(1.0, 4.0).mean();          // 2.5
// Pair::new(1, 2).mean();           // E0599: Pair<{integer}> に mean は無い
// Pair::new(1, "x");                // E0308: a と b は同じ T
```

- `impl<T>` の `<T>` は「ここで型引数 `T` を導入する」宣言。`impl Pair<f64>` は導入しないので `<>` が付かない
- `Option<T>` / `Result<T, E>` / `Vec<T>` / `HashMap<K, V>` も**ただのジェネリックな型**で、特別扱いはされていない

### 単相化（monomorphization）

ジェネリクスはコンパイル時に、使われた型ごとに**別々の関数として展開**される。

```text
largest::<i32>  → i32 専用の機械語
largest::<f64>  → f64 専用の機械語
largest::<char> → char 専用の機械語
```

- 実行時に型を調べる処理が無いので**コストはゼロ**（手で型ごとに書いたのと同じ速さ）
- 代わりに、使う型の種類が増えるほどバイナリが大きくなり、コンパイルが遅くなる
- TS のジェネリクスはコンパイル後に消える（型消去）。Rust は逆に、型ごとに増える

---

## 3. trait 境界

### 3 通りの書き方

```rust
// 1. インライン
fn join_all<T: fmt::Display>(items: &[T], sep: &str) -> String { ... }

// 2. where 句（境界が長いとき・複数の型引数があるとき）
fn count_unique<T>(items: &[T]) -> usize
where
    T: Eq + std::hash::Hash,
{ ... }

// 3. impl Trait（引数位置）: 型引数に名前を付けない短縮記法
fn total_len(words: impl IntoIterator<Item = impl AsRef<str>>) -> usize { ... }

total_len(vec!["仕訳", "試算表"]);                     // Vec<&str>
total_len(["abc".to_string(), "de".to_string()]);    // [String; 2]
```

1〜3 はどれもジェネリクス（静的ディスパッチ）で、生成されるコードは同じ。使い分け:

| 書き方 | 向いている場面 |
|---|---|
| `<T: Bound>` | 短い境界。`T` を複数の引数・戻り値で共有したいとき |
| `where` | 境界が複数行になるとき。読みやすさ優先 |
| `impl Trait` | 引数 1 つにだけ関係する境界。呼び出し側で `::<>` を使わないとき |

複数の引数が**同じ型**である必要があるなら `<T>` を使う。`fn f(a: impl Display, b: impl Display)` は `a` と `b` が別の型でもよい。

### `impl Trait`（戻り値位置）

「具体的な型は言わないが、この trait を実装した**何か 1 つの型**を返す」。

```rust
fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
    // 本当の型は Filter<Range<u32>, {closure}>。長い上にクロージャの型には名前が無い
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

- クロージャやイテレータ連鎖を返すときは事実上これが必須
- 中身は**1 つの具体型**でなければならない。`if` と `else` で別の型を返すと `E0308`。分岐で型を変えたいなら `Box<dyn Trait>`（→ 4 章）

### 条件付き実装

`impl` ブロック自体に境界を付けると、「`T` がその trait を満たすときだけ」メソッドが生える。

```rust
impl<T: fmt::Display + PartialOrd> Pair<T> {
    fn show_larger(&self) -> String { ... }
}

Pair::new(3, 8).show_larger();       // OK: i32 は Display + PartialOrd
// Pair::new(Circle {..}, Circle {..}).show_larger(); // E0599: Circle は満たさない
```

`Vec<T>` が `T: PartialEq` のときだけ `==` で比較できる、`T: Debug` のときだけ `{:?}` で表示できる、というのもこの仕組み。

### ブランケット実装

「ある trait を実装する**全ての型**」に別の trait を一括で実装する。

```rust
trait Label { fn label(&self) -> String; }

impl<T: fmt::Debug> Label for T {
    fn label(&self) -> String { format!("<<{self:?}>>") }
}

42.label();          // "<<42>>"
[1, 2].label();      // "<<[1, 2]>>"
```

標準ライブラリの代表例:

| ブランケット実装 | 結果 |
|---|---|
| `impl<T: Display + ?Sized> ToString for T` | `Display` を実装すれば `to_string()` が使える |
| `impl<T, U: From<T>> Into<U> for T` | `From` を実装すれば `into()` が使える |
| `impl<I: Iterator> IntoIterator for I` | イテレータはそのまま `for` に渡せる |

`?Sized` は「サイズが決まっていない型（`str` や `dyn Trait`）も含む」という意味。境界の `T` はデフォルトで `Sized`（コンパイル時にサイズが分かる型）に限られる。

---

## 4. 静的ディスパッチと動的ディスパッチ（`dyn Trait`）

trait を使った多態には 2 種類ある。

```rust
// 静的ディスパッチ: S ごとに関数が生成される
fn total_area_static<S: Shape>(shapes: &[S]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// 動的ディスパッチ: 実行時に vtable を引いて area を呼ぶ
fn total_area_dyn(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

let rects = [Rect { .. }, Rect { .. }];         // 全部同じ型 → 静的で足りる
total_area_static(&rects);

// let mixed = [Circle { .. }, Rect { .. }];    // E0308: 配列の要素は同じ型
let mixed: Vec<Box<dyn Shape>> = vec![          // 型の違う値を 1 つに → dyn が必要
    Box::new(Circle { r: 1.0 }),
    Box::new(Rect { w: 2.0, h: 3.0 }),
];
total_area_dyn(&mixed);
```

| | ジェネリクス `<T: Trait>` / `impl Trait` | `dyn Trait` |
|---|---|---|
| どの実装を呼ぶか決まる時点 | コンパイル時 | 実行時（vtable 経由） |
| 実行コスト | 0（インライン化もされる） | 間接呼び出し 1 回。インライン化されない |
| バイナリサイズ | 型の数だけ増える | 1 つで済む |
| 型の違う値を混ぜる | できない | できる（`Vec<Box<dyn Trait>>`） |
| 置き場所 | そのまま（スタックに置ける） | `&dyn` / `Box<dyn>` / `Rc<dyn>` などポインタ越しのみ |

### `dyn Trait` はなぜポインタ越しなのか

`dyn Shape` は「`Shape` を実装した**何か**」なので、`Circle` か `Rect` かによってサイズが違う。サイズが分からない値は変数に直接置けないため、必ずポインタの後ろに置く。

そのポインタは**データへのポインタ + vtable（メソッド表）へのポインタ**の 2 つ組（fat pointer）になる。

```rust
std::mem::size_of::<&Rect>();       // 8（64bit 環境）
std::mem::size_of::<&dyn Shape>();  // 16 = データ 8 + vtable 8
```

```text
&dyn Shape ──┬─ data ───▶ Circle { r: 1.0 }
             └─ vtable ─▶ [ drop, size, align, area, name, describe ]
```

関数の引数として受けるだけなら `Box` は要らない。`&dyn Shape` で受ければ、所有権を奪わずにどの `Shape` でも渡せる。

```rust
fn print_shape(s: &dyn Shape) { println!("{}", s.describe()); }
```

### dyn にできない trait（dyn compatibility）

vtable に載せられないメソッドを持つ trait は `dyn` にできない（`E0038`。以前は "object safety" と呼ばれていた）。主な条件:

| `dyn` にできない理由 | 例 | 理由 |
|---|---|---|
| `Self` を値で返す | `fn from_raw(v: i64) -> Self` | 呼び出し側は `Self` のサイズを知らない |
| `self` を取らない関連関数 | `fn new() -> Self` | どの型の関数を呼ぶか決める手がかりが無い |
| ジェネリックなメソッド | `fn visit<T>(&self, t: T)` | `T` ごとに無限に実装があり得るので表を作れない |
| 関連定数 | `const SYMBOL: &str` | vtable は関数しか載せられない |

```rust
// let u: Box<dyn Unit> = Box::new(Yen(1)); // E0038: the trait `Unit` is not dyn compatible
```

逃げ道として、そのメソッドだけに `where Self: Sized` を付けると「`dyn` 経由では呼べないが、trait 自体は `dyn` にできる」状態にできる。

### 使い分けの目安

- **基本はジェネリクス**。速く、コンパイラの検査も強い
- 「**異なる型を 1 つのコレクションに入れたい**」「**実行時の設定で実装を切り替えたい**」「**プラグインのように後から種類が増える**」なら `dyn`
- エラー型の `Box<dyn Error>`（S4）は、「どんなエラーでも入る」ために `dyn` を使っている例

---

## 5. 関連型と標準 trait

### 関連型（associated type）

trait の中で「実装側が 1 つに決める型」を宣言する。`Iterator` の `Item` が代表例。

```rust
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;                          // このイテレータは u32 を返す

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 { None } else { self.0 -= 1; Some(self.0 + 1) }
    }
}

let v: Vec<u32> = Countdown(5).collect();     // [5, 4, 3, 2, 1]
let s: u32 = Countdown(10).filter(|n| n % 2 == 0).sum();
for n in Countdown(3) { ... }                 // for にも渡せる
```

**関連型と型引数の違い**:

| | 関連型 `trait Iterator { type Item; }` | 型引数 `trait From<T>` |
|---|---|---|
| 1 つの型に対する実装の数 | 1 つだけ（`Countdown` の `Item` は `u32` に決まる） | 何個でも（`String: From<&str>` も `From<char>` も） |
| 使う側の書き方 | `I: Iterator<Item = u32>` | `T: From<&str>` |
| 向いている場面 | 「入力が決まれば出力の型も決まる」 | 「いろいろな型から変換できる」 |

`Add` の `type Output` も関連型（`a + b` の結果の型は `a` と `b` の型で決まる）。

### よく使う標準 trait

| trait | できるようになること | derive | 備考 |
|---|---|---|---|
| `Debug` | `{:?}` で表示 | ○ | 開発者向け。ほぼ全ての型に付ける |
| `Display` | `{}` で表示、`to_string()` | × | ユーザー向け。文言は自分で決める |
| `Clone` | `.clone()` で明示的に複製 | ○ | |
| `Copy` | 代入・引数渡しで move ではなくコピー | ○ | `Clone` も必要。ヒープを持つ型（`String` / `Vec`）には付けられない |
| `PartialEq` / `Eq` | `==` / `!=` | ○ | `f64` は `NaN != NaN` なので `PartialEq` のみ |
| `PartialOrd` / `Ord` | `<` / `>`、`sort()`、`BTreeMap` のキー | ○ | derive の比較順はフィールド（`enum` なら variant）の宣言順 |
| `Hash` | `HashMap` / `HashSet` のキー | ○ | `Eq` とセットで |
| `Default` | `Default::default()`、`..Default::default()` | ○ | 数値は 0、`String` は空、`bool` は `false` |
| `From` / `Into` | 型変換。`?` でのエラー変換（S4） | × | `From` を実装すれば `Into` は自動 |
| `TryFrom` / `TryInto` | 失敗し得る変換（`Result` を返す） | × | `u8::try_from(300_i32)` は `Err` |
| `FromStr` | `"..".parse::<T>()` | × | |
| `Iterator` / `IntoIterator` | `for` ループ、イテレータメソッド | × | |
| `Add` / `Sub` / `Mul` … | 演算子 `+` / `-` / `*` … | × | `std::ops` |
| `Drop` | スコープを抜けたときの後始末 | × | S1 の drop で呼ばれる |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct AccountCode(u16);

impl fmt::Display for AccountCode {           // Display だけは手で書く
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:04}", self.0)
    }
}

let a = AccountCode(101);
let b = a;                  // Copy なので a はまだ使える
codes.sort();               // Ord
names.insert(a, "現金");    // Eq + Hash

let cfg = Config { retries: 3, ..Default::default() };   // 残りはデフォルト値
```

derive は「全フィールドがその trait を実装していれば」使える。フィールドに `f64` があると `Eq` / `Ord` / `Hash` は derive できない。

### `From` / `Into`

```rust
impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}

let f1 = Fahrenheit::from(Celsius(100.0));
let f2: Fahrenheit = Celsius(-40.0).into();   // 代入先の型から変換先が決まる
```

- 実装するのは `From`、使うときは読みやすい方（関数の引数では `impl Into<String>` を受けると呼び出し側が楽）
- 情報が失われない変換（`i32` → `i64`）は `From`、失われ得る変換（`i32` → `u8`）は `TryFrom`。`as` キャストは黙って切り捨てるので、アプリのコードでは `TryFrom` を優先する

### 演算子オーバーロード

`a + b` は `Add::add(a, b)` の糖衣構文。

```rust
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { Vec2 { x: self.x + rhs.x, y: self.y + rhs.y } }
}
```

`+=` は別の trait（`AddAssign`）。`iter().sum()` を使えるようにするには `Sum` を実装する（→ 8 章の `Money`）。

---

## 6. orphan rule と newtype

**orphan rule（孤児ルール）**: `impl Trait for Type` は、**trait か型の少なくとも一方が自分の crate で定義されたもの**でなければならない。

| trait | 型 | 実装できるか | 例 |
|---|---|---|---|
| 自分の | 自分の | ○ | `impl Shape for Rect` |
| 自分の | 外部の | ○ | `impl Double for i32` |
| 外部の | 自分の | ○ | `impl Display for Money` |
| 外部の | 外部の | **×**（E0117） | `impl Display for Vec<String>` |

理由: もし 2 つの crate がそれぞれ `impl Display for Vec<String>` を書けたら、両方を使うプログラムでどちらを使うか決められない。このルールで「ある型のある trait の実装は世界に 1 つ」が保証される。

### newtype パターン

外部の型を、フィールド 1 つのタプル構造体で包んで「自分の型」にする。

```rust
struct Lines(Vec<String>);

impl fmt::Display for Lines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, line) in self.0.iter().enumerate() {
            writeln!(f, "{:>3}| {line}", i + 1)?;
        }
        Ok(())
    }
}
```

- 実行時のコストは無い（`Lines` と `Vec<String>` はメモリ上同じ）
- 包んだ型のメソッドは使えなくなるので、必要なものは自分で委譲するか `.0` で取り出す
- orphan rule の回避以外に、**意味の違う値を型で区別する**用途がある（`Money(i64)` と `AccountId(i64)` を取り違えるとコンパイルエラーになる）。こちらの用途の方が重要（→ 8 章）

---

## 7. よくあるコンパイルエラーと直し方

エラー番号の詳細は `rustc --explain E0277` で読める。

| エラー | メッセージ（要約） | 原因 | 典型的な直し方 |
|---|---|---|---|
| E0277 | the trait bound `String: Copy` is not satisfied | 境界を満たさない型を渡した | 境界を緩める（`Copy` → `Clone` にして `.clone()`、参照を返す）／型側に `impl` か `derive` を足す |
| E0277 | `Circle` doesn't implement `Debug` | `{:?}` で表示しようとした | `#[derive(Debug)]` を付ける |
| E0369 | binary operation `>` cannot be applied to type `T` | ジェネリック関数で `T` に境界が無い | `T: PartialOrd` を足す（`+` なら `T: Add<Output = T>`） |
| E0599 | no method named `foo` found for type parameter `T` | 境界に無いメソッドを `T` で呼んだ | 境界にその trait を足す |
| E0599 | no method named `foo` found … items from traits can only be used if the trait is in scope | trait が `use` されていない | エラーが示す `use std::io::Write;` などを足す |
| E0046 | not all trait items implemented, missing: `area` | 必須メソッドの書き忘れ | メソッドを実装する |
| E0117 | only traits defined in the current crate can be implemented for types defined outside of the crate | orphan rule 違反 | newtype で包む／自分の trait を定義する |
| E0038 | the trait `Unit` is not dyn compatible | `Self` を返す・ジェネリックメソッドなどを持つ trait を `dyn` にした | ジェネリクスにする／該当メソッドに `where Self: Sized`／trait を分割する |
| E0308 | `if` and `else` have incompatible types（戻り値が `impl Trait`） | `impl Trait` で別々の型を返した | `Box<dyn Trait>` を返す／`enum` で包む |
| E0282 / E0283 / E0284 | type annotations needed | `collect` / `parse` / `into` などの変換先が推論できない | `::<T>` か `let x: T` で型を書く |
| E0782 | expected a type, found a trait（`dyn` を付けて） | `Box<Shape>` のように `dyn` を書き忘れた | `Box<dyn Shape>` |
| E0689 | can't call method `double` on ambiguous numeric type `{integer}` | 数値リテラルの型が決まらないまま自作 trait のメソッドを呼んだ | `21_i32.double()` のように型を付ける |

読み方のコツ: E0277 は「必要な trait が実装されていない」を表す最頻出のエラー。メッセージの `required by a bound in ...` の行に**どの関数のどの境界が要求したか**が書いてあるので、そこから「境界を緩めるか、型に実装を足すか」を考える。

---

## 8. 設計の指針

- **まずは具体的な型で書き、2 つ目の使い道が出てから一般化する**: 最初からジェネリクスにすると境界が増えて読みにくくなる。重複が見えてから trait を切り出しても遅くない
- **trait は小さく**: 1 つの trait に多くのメソッドを詰めると、実装する側の負担が増え、`dyn` にもしにくくなる。`Read` / `Write` / `Display` のように 1〜2 個の必須メソッドが理想
- **基本はジェネリクス、混ぜたい・切り替えたいときだけ `dyn`**: 4 章の表のとおり
- **引数は広く受け取る**: `&Vec<String>` より `&[String]`、さらに `impl IntoIterator<Item = impl AsRef<str>>`。呼び出し側の選択肢が増える。ただし広げすぎるとシグネチャが読めなくなるので、公開 API の境界で考える
- **標準 trait を実装して「普通の型」にする**: `Debug` は常に、値として比較・ソートしたいなら `PartialEq` / `Ord`、表示するなら `Display`。標準 trait に乗ると、`sort` / `sum` / `HashMap` / `{}` など既存の道具がそのまま使える
- **意味の違う数値は newtype で区別する**: 金額・数量・ID を全部 `i64` にすると、取り違えてもコンパイラは何も言わない

最後の 2 点はステージ2（仕訳・試算表アプリ）でそのまま使う。金額を `Money` 型にし、標準 trait を実装して集計を素直に書けるようにするスケッチ:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Money(i64);   // 円単位の整数（浮動小数の誤差を避ける）

impl Add for Money { type Output = Money; fn add(self, rhs: Money) -> Money { Money(self.0 + rhs.0) } }
impl AddAssign for Money { fn add_assign(&mut self, rhs: Money) { self.0 += rhs.0; } }
impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::default(), |acc, m| acc + m)
    }
}
impl fmt::Display for Money { ... }   // "1,234,567円"。f.pad() で幅指定 {:>12} に対応

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Account { Cash, Purchases, AccountsPayable, Sales }   // Ord → BTreeMap で宣言順に並ぶ

/// Vec でもスライスでもイテレータでも受け取れる試算表の集計
fn trial_balance<'a, I>(lines: I) -> Vec<(Account, Money, Money)>
where
    I: IntoIterator<Item = &'a Line>,
{
    let mut map: BTreeMap<Account, (Money, Money)> = BTreeMap::new();
    for l in lines {
        let e = map.entry(l.account).or_default();   // (Money, Money) が Default だから使える
        match l.side {
            Side::Debit => e.0 += l.amount,           // AddAssign
            Side::Credit => e.1 += l.amount,
        }
    }
    map.into_iter().map(|(a, (d, c))| (a, d, c)).collect()
}

/// 出力形式は実行時に選ぶ → dyn
trait Render { fn render(&self, rows: &[(Account, Money, Money)]) -> String; }
fn pick_renderer(format: &str) -> Box<dyn Render> {
    match format {
        "csv" => Box::new(CsvRender),
        _ => Box::new(TextRender),
    }
}

let total: Money = rows.iter().map(|r| r.1).sum();   // Sum
// Money(1) + 1;   // E0308: 素の i64 とは足せない
```

ここでは「集計」はジェネリクス（入力の型はコンパイル時に決まる）、「出力形式」は `dyn`（コマンドライン引数で実行時に決まる）と、4 章の使い分けをそのまま当てはめている。動く全体は `exercises/traits/src/main.rs` の `d8_trial_balance`。

---

## 9. 理解度チェック

答えを見ずに、自分の言葉で説明できるか確かめる（マトリクスの S5 を埋める前に）。

- [ ] TS の interface（構造的）と Rust の trait（名目的）の違い。Rust の方式の利点と欠点を 1 つずつ
- [ ] デフォルト実装があると、実装する側は何が楽になるか。`Iterator` を例に
- [ ] `&self` / `&mut self` / `self` のメソッドをそれぞれ 1 つずつ標準ライブラリから挙げる
- [ ] 関連関数 `fn new() -> Self` を trait に置けることで、何ができるようになるか
- [ ] `fn largest<T>(list: &[T]) -> T` の中で `>` が使えない理由と、直し方
- [ ] 単相化とは何か。実行速度とバイナリサイズにどう影響するか
- [ ] `<T: Display>` / `where T: Display` / `impl Display` の違い（または同じ点）
- [ ] `fn f(a: impl Display, b: impl Display)` と `fn f<T: Display>(a: T, b: T)` は何が違うか
- [ ] 戻り値の `impl Iterator` で、`if` と `else` で別のイテレータを返せない理由と、回避策
- [ ] `Vec<Box<dyn Shape>>` が必要になる場面と、`Vec<S>`（`S: Shape`）で足りる場面
- [ ] `&dyn Shape` のサイズが `&Rect` の 2 倍になる理由
- [ ] `trait Unit { fn from_raw(v: i64) -> Self; }` が `dyn Unit` にできない理由
- [ ] 関連型（`type Item`）と型引数（`From<T>`）の使い分け
- [ ] `#[derive(Eq, Hash)]` を付けた struct に `f64` のフィールドを足すと何が起きるか、なぜか
- [ ] `From` を実装すると `Into` が使える仕組み（ブランケット実装）
- [ ] orphan rule が無いと何が困るか。newtype でどう回避するか
- [ ] 金額を `i64` ではなく `Money` 型にすると、試算表アプリのどんなバグが防げるか
