# 所有権・借用・ライフタイム（S1〜S3 深掘り）

> `rust-syntax-overview.md` の S1〜S3 を詳しく掘り下げたリファレンス。
> サンプルコードは `exercises/ownership/src/main.rs` に関数単位でまとめてあり、`cargo run -p ownership` でそのまま動く。
> エラーになる例はコメントアウトしてあるので、外してエラーメッセージを実際に読むこと。
> マトリクス（`rust-3layer-matrix.md`）の S1〜S3 には**このファイルを写さず**、自分の言葉で書くこと。

---

## 目次

0. [前提: スタックとヒープ](#0-前提-スタックとヒープ)
1. [S1 所有権](#s1-所有権ownership)
2. [S2 借用](#s2-借用borrowing)
3. [S3 ライフタイム](#s3-ライフタイムlifetime)
4. [よくあるコンパイルエラーと直し方](#4-よくあるコンパイルエラーと直し方)
5. [設計の指針](#5-設計の指針)
6. [理解度チェック](#6-理解度チェック)

---

## 0. 前提: スタックとヒープ

所有権は「**ヒープ上のメモリを誰が、いつ解放するか**」を決める仕組み。まず 2 つの置き場所を区別する。

| | スタック | ヒープ |
|---|---|---|
| 置けるもの | サイズがコンパイル時に決まる値 | 実行時にサイズが変わる値 |
| 確保・解放 | 関数の出入りで自動（速い） | 明示的に確保・解放が必要（遅い） |
| 例 | `i32`、`bool`、`[i32; 3]`、`String` の**本体** | `String` の**文字データ**、`Vec` の要素 |

`String` はスタック上の 3 ワード（`ptr` / `len` / `capacity`）と、`ptr` が指すヒープ上のバイト列でできている。

```text
スタック                      ヒープ
┌──────────┬────┐            ┌───┬───┐
│ ptr      │ ───┼──────────▶ │ h │ i │ ...
│ len      │ 2  │            └───┴───┘
│ capacity │ 4  │
└──────────┴────┘
```

```rust
println!("{}", std::mem::size_of::<String>()); // 24（64bit 環境で 8 バイト × 3）

let mut s = String::with_capacity(4);
s.push_str("hi");        // len=2, capacity=4
s.push_str(" there");    // 容量を超えたのでヒープを再確保。ptr が変わることがある
```

**JS との違い**: JS ではヒープ上のオブジェクトを GC が「もう誰からも参照されていない」と判断したときに回収する。Rust には GC が無いので、**解放のタイミングをコンパイル時に一意に決める**必要がある。そのための規則が所有権。

---

## S1 所有権（Ownership）

### 3 つのルール

1. 各値には**所有者（owner）がちょうど 1 つ**
2. 所有者がスコープを抜けると値は**自動で破棄（drop）**される
3. 所有権は代入・引数渡し・戻り値で**ムーブ（移動）**する

ルール 1 があるからルール 2 の「解放」が 1 回だけになる（二重解放が起きない）。

### ムーブ

```rust
let s1 = String::from("hello");
let s2 = s1;             // ptr / len / capacity だけがコピーされ、s1 は無効になる
// println!("{s1}");     // error[E0382]: borrow of moved value: `s1`
```

もし `s1` も有効なままだと、スコープ終端で `s1` と `s2` の両方が同じヒープを解放してしまう（二重解放）。だから「コピーしたうえで元を無効にする」= ムーブ。

関数との受け渡しも同じ規則:

```rust
fn takes(s: String) -> usize { s.len() }   // s はここで drop される
fn gives() -> String { String::from("x") } // 戻り値として呼び出し元へムーブ
fn takes_and_gives_back(s: String) -> String { s }

let a = String::from("abc");
let n = takes(a);
// println!("{a}");      // error[E0382]: a はもう takes に渡した
let c = takes_and_gives_back(gives());
```

「渡して返してもらう」は面倒なので、普通は後述の**借用**を使う。

### `Copy` と `Clone`

```rust
#[derive(Debug, Clone, Copy)]
struct Point { x: i32, y: i32 }

let p1 = Point { x: 1, y: 2 };
let p2 = p1;             // Copy 型なのでムーブではなくコピー。p1 も使える

let v1 = vec![1, 2, 3];
let v2 = v1.clone();     // Vec は Copy にできない。明示的に clone
```

| | `Copy` | `Clone` |
|---|---|---|
| 発生 | 代入・引数渡しで**暗黙に** | `.clone()` で**明示的に** |
| 付けられる型 | 全フィールドが `Copy` で、`Drop` を実装していない型 | ほぼ何でも |
| 例 | 整数、`f64`、`bool`、`char`、`&T`、それらのタプル・配列 | `String`、`Vec<T>`、`HashMap` |
| コスト | ビットコピーのみ | 任意（ヒープの複製が起きうる） |

- `Copy` を付けるには `Clone` も必要（`#[derive(Clone, Copy)]`）
- `&mut T` は `Copy` **ではない**（可変参照が 2 つに増えると借用規則が壊れるため）

### `Drop` と drop の順序

`Drop` trait を実装すると、drop 時に任意の処理を走らせられる（C++ のデストラクタ、ファイルやロックの自動解放に使われる）。

```rust
struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) { println!("drop {}", self.0); }
}

let _a = Noisy("a");
let _b = Noisy("b");
{
    let _inner = Noisy("inner");
}                            // ← "drop inner"
let c = Noisy("c");
drop(c);                     // ← "drop c"（std::mem::drop で早めに drop）
let _ = Noisy("underscore"); // ← "drop underscore"（`_` は束縛しないので即 drop）
// 関数の終わりで "drop b" → "drop a"（宣言の逆順）
```

- 変数は**宣言の逆順**で drop される
- `let _x = ...` は変数に束縛される（スコープ終端まで生きる）が、`let _ = ...` は**束縛されず即 drop**。ロックガードで踏みやすい落とし穴
- `x.drop()` を直接呼ぶことはできない（二重 drop を防ぐため）。`drop(x)` を使う

### 部分ムーブ

構造体の一部のフィールドだけをムーブできる。

```rust
struct User { name: String, age: u32 }

let u = User { name: String::from("Alice"), age: 30 };
let name = u.name;           // name だけムーブ
println!("{}", u.age);       // age は Copy なのでまだ使える
// let u2 = u;               // error[E0382]: use of partially moved value: `u`
```

**なぜあるか**: GC 無しで「いつ解放するか」を静的に決めるため。二重解放・解放後使用（use-after-free）・解放忘れの大部分がコンパイル時に排除される。C++ の RAII とムーブセマンティクスを、**デフォルト**かつ**言語規則として強制**したもの（C++ はコピーがデフォルトで、ムーブ後の変数も使えてしまう）。

---

## S2 借用（Borrowing）

所有権を渡さずに値を使わせるのが**借用**。参照（`&T` / `&mut T`）を作ることを「借用する」と言う。

### 借用規則

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;                 // 不変参照はいくつでも
println!("{r1} {r2}");

let r3 = &mut s;             // 可変参照は 1 つだけ
r3.push_str(" world");
// let r4 = &mut s;          // error[E0499]: cannot borrow `s` as mutable more than once at a time
// println!("{r3} {r4}");
```

ある時点で成り立つのは**どちらか一方**:

- 不変参照 `&T` が**いくつでも**
- 可変参照 `&mut T` が**ただ 1 つ**（このとき不変参照も持てない）

さらに、**借用されている間は元の値をムーブできない**:

```rust
let t = String::from("x");
let rt = &t;
// let t2 = t;               // error[E0505]: cannot move out of `t` because it is borrowed
println!("{rt}");
```

### NLL（Non-Lexical Lifetimes）

借用は**スコープの終わりではなく、最後に使われた位置で終わる**。

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
// v.push(4);                // error[E0502]: cannot borrow `v` as mutable
                             //   because it is also borrowed as immutable
println!("{first}");         // ← first の最後の使用。ここで借用が終わる
v.push(4);                   // OK
```

上の `push` が禁止される理由: `push` で容量が足りなくなると `Vec` はヒープを再確保して要素を移動する。すると `first` は解放済みメモリを指すダングリング参照になる。C++ の「イテレータ無効化」バグそのもの。

### 参照の自動変換（Deref 型強制）

関数の引数は `&String` / `&Vec<T>` ではなく `&str` / `&[T]` で受けるのが定石。`&String` → `&str`、`&Vec<T>` → `&[T]` はコンパイラが自動変換するので、どちらでも渡せる。

```rust
fn count_chars(s: &str) -> usize { s.chars().count() }
fn sum(xs: &[i32]) -> i32 { xs.iter().sum() }

let owned = String::from("所有権");
count_chars(&owned);         // &String → &str
count_chars("literal");      // &'static str

let v = vec![1, 2, 3];
let arr = [4, 5, 6];
sum(&v);                     // &Vec<i32> → &[i32]
sum(&arr);                   // &[i32; 3] → &[i32]
```

### スライス

スライスは「**連続したデータの一部**への参照」。中身は `ptr` + `len`（ファットポインタ）。

```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

let mut s = String::from("hello world");
let w = first_word(&s);      // w は s の中を指している
// s.clear();                // error[E0502]: w が生きている間は s を変更できない
println!("{w}");

let a = [10, 20, 30, 40];
let mid = &a[1..3];          // [20, 30]
```

JS で `first_word` が「先頭の単語の終了位置（数値）」を返す設計だと、元の文字列が変わっても数値は古いまま残り、バグに気付けない。スライスを返せば、その不整合をコンパイラが検出する。

### メソッドの `self` / `&self` / `&mut self`

| レシーバ | 意味 | 呼んだ後の値 | 典型例 |
|---|---|---|---|
| `&self` | 不変借用 | そのまま使える | ゲッター、`len()`、`iter()` |
| `&mut self` | 可変借用 | そのまま使える（`let mut` が必要） | `push()`、`inc()` |
| `self` | 所有権を受け取る（消費） | **使えなくなる** | `into_*()`、ビルダーの `build()` |

```rust
impl Counter {
    fn get(&self) -> u32 { self.count }
    fn inc(&mut self) { self.count += 1; }
    fn into_name(self) -> String { self.name }
}

let mut c = Counter::new("clicks");
c.inc();
let name = c.into_name();
// c.get();                  // error[E0382]: borrow of moved value: `c`
```

- `obj.method()` の呼び出し側は `&` / `&mut` を書かなくてよい（自動参照）
- 命名規約: `as_*` は安い借用変換、`to_*` は複製を伴う変換、`into_*` は所有権を消費する変換

**なぜあるか**: 「共有」と「変更」を同時に許さない（aliasing XOR mutability）。これだけでイテレータ無効化・データ競合（複数スレッドからの同時書き込み）をコンパイル時に排除できる。JS はシングルスレッドで GC もあるので、この種の問題を言語レベルで意識する必要がない。

---

## S3 ライフタイム（Lifetime）

**ライフタイム**は「参照が有効な期間」。コンパイラ（借用チェッカー）は、**参照が参照先より長生きしない**ことを検証する。

```rust
let r;
{
    let x = 5;
    r = &x;                  // error[E0597]: `x` does not live long enough
}                            // x はここで drop
println!("{r}");             // r はもう存在しない x を指している
```

### ライフタイム注釈

関数が複数の参照を受け取って参照を返すとき、**戻り値がどの引数から借りているか**をコンパイラは関数のシグネチャだけからは判断できない。そこで注釈で説明する。

```rust
// 「戻り値は x と y の両方が有効な期間だけ有効」という宣言
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

let a = String::from("long string");
let result;
{
    let b = String::from("xyz");
    result = longest(&a, &b); // error[E0597]: `b` does not live long enough
}
println!("{result}");         // 'a は a と b の短い方（b）に合わせられる
```

- 注釈は**寿命を変えない**。複数の参照の関係をコンパイラに説明するだけ
- `'a` という名前に意味はない（慣習で `'a`, `'b` …）
- 借用チェッカーは**関数ごと**にシグネチャだけを見て検証する。だから関数の中身を読まずに呼び出し側を検査できる

### ライフタイム省略規則

多くの関数で注釈を書かなくて済むのは、次の 3 規則でコンパイラが補うから。補えない場合だけ注釈が必要。

1. 参照の引数には、それぞれ別のライフタイムが割り当てられる
2. 参照の引数が**ちょうど 1 つ**なら、戻り値の参照はそのライフタイムになる
3. 引数に `&self` / `&mut self` があれば、戻り値の参照は `self` のライフタイムになる

```rust
fn first_word(s: &str) -> &str                // 規則 2 で OK
fn longest(x: &str, y: &str) -> &str          // 引数 2 つ・self 無し → 補えない（E0106）
fn part(&self) -> &str                        // 規則 3 で OK
```

参照を返す関数で、引数に参照が 1 つも無い場合は「関数内で作った値への参照」しか返しようがなく、それはダングリングになる:

```rust
// fn dangle() -> &String {   // error[E0106]: missing lifetime specifier
//     let s = String::from("hi");
//     &s                     // s は関数の終わりで drop される
// }
fn no_dangle() -> String {     // 所有した値を返せばよい
    String::from("hi")
}
```

### 参照を持つ構造体

構造体のフィールドに参照を持たせるときは注釈が必須。「この構造体は参照先より長生きできない」という意味になる。

```rust
struct Excerpt<'a> {
    part: &'a str,
}

impl Excerpt<'_> {             // '_ は「名前は使わない」の意味
    fn part(&self) -> &str { self.part }
}

let novel = String::from("Call me Ishmael. Some years ago...");
let first = novel.split('.').next().unwrap_or("");
let e = Excerpt { part: first }; // e は novel より長生きできない
```

学習初期は、構造体には `&str` ではなく `String` を持たせる（所有させる）方が圧倒的に楽。

### `'static` と `T: 'static`

この 2 つは似ているが意味が違う。

| 書き方 | 意味 | 例 |
|---|---|---|
| `&'static T` | **プログラム終了まで有効な参照** | 文字列リテラル `"hello"`、`static` 変数への参照 |
| `T: 'static`（トレイト境界） | `T` が**短命な参照を含まない**（＝所有した値か `'static` 参照だけでできている） | `String`、`i32`、`Vec<String>`、`&'static str` |

```rust
fn needs_static<T: std::fmt::Debug + 'static>(t: T) { println!("{t:?}"); }

needs_static(String::from("owned"));  // OK: String は参照を含まない
needs_static("literal");              // OK: &'static str
// let local = String::from("x");
// needs_static(&local);              // error[E0597]: `local` does not live long enough
```

`T: 'static` は「永遠に生きる」ではなく「**生かしておこうと思えばいつまでも生かせる**」という意味。`thread::spawn` に渡すクロージャで要求される（スレッドがいつ終わるか分からないため）。

**なぜあるか**: GC が無いので「参照先がまだ生きているか」を実行時に確かめられない。そこでコンパイル時に検証する。JS では参照が生きている限り GC が参照先を回収しないので、この概念自体が不要。

---

## 4. よくあるコンパイルエラーと直し方

エラー番号の詳細は `rustc --explain E0382` で読める。

| エラー | メッセージ（要約） | 原因 | 典型的な直し方 |
|---|---|---|---|
| E0382 | borrow of moved value | ムーブ済みの値を使った | 借用で渡す（`&x`）／`clone()`／ムーブ前に使う |
| E0499 | cannot borrow as mutable more than once | `&mut` を同時に 2 つ作った | 1 つ目の使用を終えてから 2 つ目を作る／ブロックで区切る |
| E0502 | cannot borrow as mutable because it is also borrowed as immutable | 不変参照が生きている間に変更しようとした | 不変参照の最後の使用より後で変更する／必要な値をコピーしておく |
| E0505 | cannot move out because it is borrowed | 借用中の値をムーブした | 借用を先に使い終える／`clone()` |
| E0106 | missing lifetime specifier | 戻り値の参照の出どころを推論できない | ライフタイム注釈を書く／所有した値（`String`）を返す |
| E0597 | does not live long enough | 参照が参照先より長生きする | 参照先のスコープを広げる／所有した値を持たせる |

読み方のコツ: エラーメッセージは「どこで借りた（borrowed here）」「どこで動かした（moved here）」「どこでまだ使っている（later used here）」の 3 点を矢印で示している。**3 点の位置関係**を見れば原因が分かる。

---

## 5. 設計の指針

- **引数は借用で受ける**: `&str` / `&[T]` / `&T`。所有権が必要なとき（構造体に保存する、別スレッドに渡す）だけ `String` / `Vec<T>` / `T` で受ける
- **迷ったら所有した値を返す**: 参照を返すとライフタイム注釈が必要になりがち。`String` を返せば悩まない
- **構造体には所有させる**: フィールドは `&str` より `String`。参照を持つ構造体は、パーサなど性能が必要になってから
- **`clone()` は最初は使ってよい**: 借用チェッカーと格闘して止まるより、`clone()` で先に進む方が学習効率が良い。動いてから `clippy` の指摘を見て減らす
- **所有者が 1 つに決まらない設計**（グラフ、双方向リンク、共有状態）は所有権だけでは書けない。`Rc<T>`（共有所有）・`RefCell<T>`（実行時の借用チェック）を使う → `rust-syntax-overview.md` の S7

---

## 6. 理解度チェック

答えを見ずに、自分の言葉で説明できるか確かめる（マトリクスの S1〜S3 を埋める前に）。

- [ ] `let s2 = s1;` で `String` はムーブされるのに、`i32` はコピーされる。なぜ違うのか
- [ ] ムーブが無かったら（＝ `s1` も使えたら）何が起きるか
- [ ] `let _ = guard;` と `let _guard = guard;` の違い
- [ ] 「不変参照はいくつでも、可変参照は 1 つだけ」の規則が防いでいるバグを 2 つ挙げる
- [ ] `v.push()` が `&v[0]` の生存中に禁止される理由を、メモリの図で説明する
- [ ] 関数の引数を `&String` ではなく `&str` にする理由
- [ ] `fn longest(x: &str, y: &str) -> &str` がエラーで、`fn first_word(s: &str) -> &str` がエラーにならない理由
- [ ] ライフタイム注釈は寿命を延ばせるか
- [ ] `&'static str` と `T: 'static` の違い
- [ ] JS にはなぜ所有権もライフタイムも無いのか（GC との関係）
