extern crate lalrpop;
extern crate rflex;

use std::env;
use std::path::Path;


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("nix_lexer.rs");
    let path = Path::new("src").join("nix_lexer.l");
    if let Err(e) = rflex::process(path, Some(dest)) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    lalrpop::process_root().unwrap();
}
