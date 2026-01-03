use arboard::Clipboard;
use std::env::args;

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    let text = args.join(" ");
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text).unwrap();
}
