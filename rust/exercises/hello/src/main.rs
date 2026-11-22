//! 環境の動作確認用 crate。
//! `cargo run -p hello` と `cargo test` が通れば Docker 環境は正常。

fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    println!("{}", greet("Rust in Docker"));
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greet_embeds_the_name() {
        assert_eq!(greet("Rust"), "Hello, Rust!");
    }
}
