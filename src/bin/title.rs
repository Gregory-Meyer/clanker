extern crate unicode_segmentation;

mod compress;

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();

    if let Some(running) = args.get(1) {
        println!("{} {}", running, compress::compressed_cwd())
    } else {
        println!("{}", compress::compressed_cwd());
    }
}
