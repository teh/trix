extern crate lalrpop;
extern crate rflex;

use std::env;
use std::path::Path;


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("nix_lexer.rs");
    let path = Path::new("src").join("nix_lexer.l");
    if let Err(e) = rflex::process(path, Some(dest.clone())) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
    // The lexer allocates a 1MB lookup table on each iteration which is quite
    // slow. The following is a dumb hack which reduces the allocation to just
    // enough to keep track of newlines and maps the rest to 0.
    // See https://github.com/pfnet/rflex/issues/11
    let d = std::fs::read_to_string(&dest).unwrap();
    let d = d.replace("self.cmap[zz_input as usize]", "if zz_input < 0xff { self.cmap[zz_input as usize] } else { 0usize }");
    let d = d.replace("0x110000", "0x2029 + 1");
    let d = d.replace(
        "let idx = Lexer::ZZ_ROW",
        "if zz_input == 0x0a { self.zz_lineno += 1; }\nlet idx = Lexer::ZZ_ROW",
    );
    let d = d.replace(
        "zz_state: usize,",
        "zz_lineno: usize,\nzz_state: usize,",
    );
    let d = d.replace(
        "zz_state: 0,",
        "zz_lineno: 0,\nzz_state: 0,",
    );
    std::fs::write(&dest, d).unwrap();
    lalrpop::process_root().unwrap();
}
