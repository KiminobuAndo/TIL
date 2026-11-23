# Rust 学習環境（Docker）

ローカルに Rust ツールチェーンを入れず、Docker コンテナ内だけで Rust を学習するための環境。
最終的な到達目標は **仕訳・試算表アプリ**の実装（`programming_learning_framework.md` のステージ2）。

---

## 前提条件

- **Windows 側で Docker Desktop を起動しておくこと。**
  起動していないと WSL 側の `docker` が次のエラーを返す（統合設定の問題ではなく、単に Docker Desktop が動いていないだけ）:

  ```
  The command 'docker' could not be found in this WSL 2 distro.
  ```

- ローカル（WSL）には Rust / cargo / rustup を**インストールしない**。すべてコンテナ内で実行する。

---

## 使い方

すべて `til/rust/` ディレクトリで実行する。

| やること | コマンド |
|---|---|
| 初回ビルド（イメージ作成） | `docker compose build` |
| 作業シェルに入る | `docker compose run --rm rust` |
| 実行（単発） | `docker compose run --rm rust cargo run -p hello` |
| 後片付け（キャッシュも削除） | `docker compose down -v` |

基本は**シェルに入ってから cargo を普通に叩く**スタイル。コンパイルエラーとの往復が多いので、都度コンテナを起動するより速い。

```bash
docker compose run --rm rust        # ← コンテナ内のシェルに入る

# 以降はコンテナ内（/workspace）で
cargo run -p hello                  # 実行
cargo test                          # テスト（workspace 全体）
cargo clippy                        # 静的解析（学習中は最も有用）
cargo fmt                           # 整形
cargo new exercises/fizzbuzz        # 課題を1つ追加する
exit                                # 抜ける（--rm なのでコンテナは破棄される）
```

### 環境の構造

```
rust/
├── Dockerfile          # rust:1-slim-bookworm + clippy / rustfmt
├── compose.yaml        # サービス rust ひとつ
├── Cargo.toml          # workspace ルート（members = ["exercises/*"]）
├── notes/              # 学習メモ
└── exercises/          # 課題を 1 課題 = 1 crate で置く
    └── hello/          # 環境の動作確認用
```

| 設計 | 理由 |
|---|---|
| ソースはホスト側（bind mount `.:/workspace`） | エディタはホストでそのまま使える |
| コンテナは uid/gid 1000 で動く | root で動かすと `cargo new` が root 所有ファイルを作り、ホストから編集できなくなる |
| ビルド成果物は named volume（`CARGO_TARGET_DIR=/cargo-target`） | リポジトリに `target/` が出ず、ビルドも速い |
| crates.io キャッシュも named volume | コンテナを捨てても再ダウンロードしない |
| Cargo workspace 構成 | 課題が増えてもビルドキャッシュとツール設定を共有できる |

`cargo new exercises/<name>` で作った crate は `members = ["exercises/*"]` のグロブに自動で含まれるため、ルートの `Cargo.toml` を編集する必要はない。

> **補足**: Rust のバージョンは Dockerfile のタグ（`rust:1-slim-bookworm`）で決まる（2026-11 時点で cargo 1.100.0）。固定したいときは `rust:1.100-slim-bookworm` のように書き換えて `docker compose build` する。`rust-toolchain.toml` は置かない方針（イメージと別バージョンを実行時にダウンロードしようとして失敗するため）。

---

## 学習ロードマップ

`programming_learning_framework.md` の4段階モデルに対応させる。Rust は「第2言語（所有権・メモリ管理を学ぶ、JS の対極）」という位置づけ。

### ステージ1: 文法の体系的習得

`notes/rust-3layer-matrix.md` を埋める。L0〜L7（言語普遍層）→ P1〜P3（パラダイム層）→ S1〜S8（Rust 固有層）の順。
「これは普遍か、Rust 固有か」をラベリングしながら進める。
参照用のまとめは `notes/rust-syntax-overview.md`（基本文法 + Rust 固有の特徴 + 他言語比較）。

### ステージ1.5: 最小実装による文法定着

1 課題 = 1 crate で `exercises/` に追加する。Rust では**各課題で所有権の壁にぶつかる**ので、「何を学ぶか」を意識して取り組む。

| # | 課題 | Rust で学ぶ中心概念 | 行数目安 |
|---|---|---|---|
| 1 | FizzBuzz | `match`、範囲 `1..=100`、`if` が式であること | 10行 |
| 2 | 配列の合計・平均・最大 | スライス `&[T]`、`Iterator`（`sum` / `max`）、整数と浮動小数の変換 | 20行 |
| 3 | 文字列の出現回数カウント | `HashMap`、`entry` API、`&str` と `String` の所有権 | 30行 |
| 4 | 簡易電卓 | `Result` とエラー処理、`parse()`、`stdin` 入力 | 50行 |
| 5 | ToDoリストのCRUD（コンソール） | `struct` / `Vec`、`&mut self`、借用規則 | 100行 |
| 6 | 簡易じゃんけんゲーム | `enum` と網羅的な `match`、`rand` crate の導入 | 80行 |
| 7 | CSV読み込み・集計 | ファイルI/O、`?` によるエラー伝播、`Iterator` 連鎖 | 150行 |
| 8 | 簡易JSON操作（パース・整形） | 再帰的 `enum`、`Box`、ライフタイム | 200行 |

進め方（フレームワーク 3.3）:

1. まず自力で書く（30分〜1時間）
2. 詰まったら AI に聞くが、**コードを丸ごとコピーしない**
3. 書けたら別のパラダイムで書き直す（手続き的 → `Iterator` チェーン等）
4. `cargo clippy` の指摘を必ず読む（Rust の慣習を学ぶ最短経路）

**達成基準**: 8課題のうち最低6つを自力実装 / 同じ課題を2つ以上の書き方で実装 / コンパイルエラーを自力で読んで直せる。

### ステージ2: 仕訳・試算表アプリ

ステージ1.5 を終えてから `app/` に crate を切る。ドメイン（複式簿記）→ 論理設計 → 実装 → 検証の順で、設計は別途行う。

想定する論点のみ先に記しておく:

- 勘定科目・貸借・仕訳の型設計（`enum` で借方/貸方、金額は整数で保持して誤差を避ける）
- 貸借一致の不変条件をどう型とコンストラクタで守るか
- 試算表の集計（`Iterator` による畳み込み）
- 永続化（まず CSV / JSON、必要になれば SQLite）

---

## Rust 固有層の論点（JS との差分）

| 概念 | Rust | JS では |
|---|---|---|
| 所有権 | 値に所有者が1つ。スコープを抜けると破棄される | GC が自動回収、所有の概念なし |
| 借用 `&` / `&mut` | 不変参照は複数可、可変参照は同時に1つだけ | 参照は自由に持てる（データ競合は言語で防げない） |
| ライフタイム `'a` | 参照の有効期間をコンパイラが検証 | 相当する概念なし |
| `Option<T>` | 値が無い状態を型で表す。`null` が存在しない | `null` / `undefined` が実行時に紛れ込む |
| `Result<T, E>` | エラーを戻り値の型で表し、`?` で伝播 | `throw` / `try-catch`（型に現れない） |
| trait | 明示的に `impl Trait for Type` と書く | 構造的型付け・プロトタイプ連鎖 |
| `match` | 網羅性をコンパイラが検査 | `switch` は網羅性を検査しない |
| 型推論 | 静的型付け + 強い推論。実行前に型が確定 | 動的型付け、暗黙変換あり（`==`） |
| 非同期 | `async` / `.await` に加えて実行ランタイムが別途必要 | イベントループが言語ランタイムに内蔵 |

---

## トラブルシュート

| 症状 | 原因と対処 |
|---|---|
| `The command 'docker' could not be found in this WSL 2 distro.` | Docker Desktop が起動していない。起動してから再実行する |
| ホストで編集できないファイルができた | コンテナを root で動かした場合に起こる。`sudo chown -R 1000:1000 .` で復旧（通常は uid 1000 で動くため発生しない） |
| ビルドが毎回遅い | `docker compose down -v` でキャッシュ volume を消していないか確認。`-v` 無しの `down` ならキャッシュは残る |
| 依存 crate の追加 | コンテナ内で `cargo add <crate>`（`-p <crate名>` で対象を指定） |
| `cargo: command not found` | `bash -l`（ログインシェル）で PATH が上書きされた場合。イメージ側で `/etc/profile.d/cargo.sh` を入れて対処済みなので、古いイメージなら `docker compose build` し直す |
| `/usr/local/cargo/registry` で `Permission denied` | キャッシュ volume が root 所有で作られている。`docker volume rm rust_cargo-registry` で消してから `docker compose build` し直す |
