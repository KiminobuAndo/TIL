//! `notes/rust-ownership-borrowing.md` のサンプルを実際に動かすための crate。
//!
//! 実行: `cargo run -p ownership`
//!
//! `// error[...]` と書いてある行はコンパイルエラーになる例。
//! コメントを外して `cargo build -p ownership` すると、そのエラー番号が出ることを確かめられる。

fn main() {
    println!("== 0. スタックとヒープ ==");
    stack_and_heap();

    println!("\n== S1 所有権 ==");
    s1_move();
    s1_function_move();
    s1_copy_clone();
    s1_drop_order();
    s1_partial_move();

    println!("\n== S2 借用 ==");
    s2_borrow_rules();
    s2_nll();
    s2_deref_coercion();
    s2_slices();
    s2_methods();

    println!("\n== S3 ライフタイム ==");
    s3_longest();
    s3_struct();
    s3_static();
}

// ---------------------------------------------------------------------------
// 0. スタックとヒープ
// ---------------------------------------------------------------------------

fn stack_and_heap() {
    // String 本体（ptr / len / capacity の 3 ワード）はスタック、文字データはヒープ
    println!("size_of::<String>() = {}", std::mem::size_of::<String>());

    let mut s = String::with_capacity(4);
    s.push_str("hi");
    println!("len={} capacity={} ptr={:p}", s.len(), s.capacity(), s.as_ptr());

    s.push_str(" there"); // 容量を超えたのでヒープを再確保する（ptr が変わることがある）
    println!("len={} capacity={} ptr={:p}", s.len(), s.capacity(), s.as_ptr());
}

// ---------------------------------------------------------------------------
// S1 所有権
// ---------------------------------------------------------------------------

fn s1_move() {
    let s1 = String::from("hello");
    let s2 = s1; // ptr / len / capacity だけがコピーされ、s1 は無効になる
    // println!("{s1}"); // error[E0382]: borrow of moved value: `s1`
    println!("s2 = {s2}");
}

fn takes(s: String) -> usize {
    s.len()
} // s はここで drop される

fn gives() -> String {
    String::from("from gives") // 戻り値として所有権を呼び出し元へムーブ
}

fn takes_and_gives_back(s: String) -> String {
    s
}

fn s1_function_move() {
    let a = String::from("abc");
    let n = takes(a);
    // println!("{a}"); // error[E0382]: borrow of moved value: `a`

    let b = gives();
    let c = takes_and_gives_back(b);
    println!("n = {n}, c = {c}");
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

fn s1_copy_clone() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = p1; // Point は Copy なのでコピー。p1 も使える
    println!("p1 = {p1:?}, p2 = {p2:?}, x + y = {}", p1.x + p2.y);

    let v1 = vec![1, 2, 3];
    let v2 = v1.clone(); // Vec は Copy にできない。明示的に clone する
    println!("v1 = {v1:?}, v2 = {v2:?}");
}

struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

fn s1_drop_order() {
    let _a = Noisy("a");
    let _b = Noisy("b");
    {
        let _inner = Noisy("inner");
    } // ← inner はここで drop
    let c = Noisy("c");
    drop(c); // std::mem::drop で明示的に早く drop できる
    let _ = Noisy("underscore"); // `_` には束縛されないので即 drop
    println!("  関数の終わり");
} // ← 宣言の逆順で b → a の順に drop

struct User {
    name: String,
    age: u32,
}

fn s1_partial_move() {
    let u = User {
        name: String::from("Alice"),
        age: 30,
    };
    let name = u.name; // name フィールドだけムーブ（部分ムーブ）
    println!("name = {name}, age = {}", u.age); // age は u32 (Copy) なのでまだ使える
    // let u2 = u; // error[E0382]: use of partially moved value: `u`
}

// ---------------------------------------------------------------------------
// S2 借用
// ---------------------------------------------------------------------------

fn s2_borrow_rules() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s; // 不変参照はいくつでも
    println!("r1 = {r1}, r2 = {r2}");

    let r3 = &mut s; // r1 / r2 はもう使われないので可変借用できる
    r3.push_str(" world");
    // let r4 = &mut s; // error[E0499]: cannot borrow `s` as mutable more than once at a time
    // println!("{r3} {r4}");
    println!("r3 = {r3}");

    let t = String::from("moved?");
    let rt = &t;
    // let t2 = t; // error[E0505]: cannot move out of `t` because it is borrowed
    println!("rt = {rt}");
}

fn s2_nll() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    // v.push(4); // error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
    println!("first = {first}"); // first の最後の使用 = 借用の終わり
    v.push(4); // ここなら OK
    println!("v = {v:?}");
}

fn count_chars(s: &str) -> usize {
    s.chars().count()
}

fn sum(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

fn s2_deref_coercion() {
    let owned = String::from("所有権");
    println!("count_chars(&String) = {}", count_chars(&owned)); // &String → &str に自動変換
    println!("count_chars(&str)    = {}", count_chars("literal"));

    let v = vec![1, 2, 3];
    let arr = [4, 5, 6];
    println!("sum(&Vec) = {}, sum(&[i32; 3]) = {}", sum(&v), sum(&arr));
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn s2_slices() {
    let mut s = String::from("hello world");
    let w = first_word(&s); // w は s の一部を指すスライス
    // s.clear(); // error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
    println!("first_word = {w}");
    s.clear();
    println!("cleared, len = {}", s.len());

    let a = [10, 20, 30, 40];
    let mid = &a[1..3];
    println!("&a[1..3] = {mid:?}");
}

struct Counter {
    name: String,
    count: u32,
}

impl Counter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
        }
    }

    fn get(&self) -> u32 {
        self.count // &self: 読むだけ
    }

    fn inc(&mut self) {
        self.count += 1; // &mut self: 変更する
    }

    fn into_name(self) -> String {
        self.name // self: 自分自身を消費する（呼んだ後は使えない）
    }
}

fn s2_methods() {
    let mut c = Counter::new("clicks");
    c.inc();
    c.inc();
    println!("count = {}", c.get());
    let name = c.into_name();
    // c.get(); // error[E0382]: borrow of moved value: `c`
    println!("name = {name}");
}

// ---------------------------------------------------------------------------
// S3 ライフタイム
// ---------------------------------------------------------------------------

// 戻り値は x と y の「両方が有効な期間」だけ有効、という宣言
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

// fn dangle() -> &String { // error[E0106]: missing lifetime specifier
//     let s = String::from("hi");
//     &s
// }

fn s3_longest() {
    let a = String::from("long string");
    {
        let b = String::from("xyz");
        let result = longest(&a, &b);
        println!("longest = {result}");
    }

    // let result;
    // {
    //     let b = String::from("xyz");
    //     result = longest(&a, &b); // error[E0597]: `b` does not live long enough
    // }
    // println!("{result}");

    // let r;
    // {
    //     let x = 5;
    //     r = &x; // error[E0597]: `x` does not live long enough
    // }
    // println!("{r}");
}

struct Excerpt<'a> {
    part: &'a str,
}

impl Excerpt<'_> {
    // 省略規則 3: &self があるので戻り値は self と同じライフタイム
    fn part(&self) -> &str {
        self.part
    }
}

fn s3_struct() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first = novel.split('.').next().unwrap_or("");
    let e = Excerpt { part: first }; // e は novel より長生きできない
    println!("excerpt = {}", e.part());
}

fn needs_static<T: std::fmt::Debug + 'static>(t: T) {
    println!("needs_static: {t:?}");
}

fn s3_static() {
    let s: &'static str = "リテラルは &'static str";
    println!("{s}");

    needs_static(String::from("所有した String は T: 'static を満たす"));
    needs_static(s);
    // let local = String::from("x");
    // needs_static(&local); // error[E0597]: `local` does not live long enough
}
