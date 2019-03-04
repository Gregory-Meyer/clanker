extern crate unicode_segmentation;

mod compress;

fn main() {
    println!("{}", compress::compressed_cwd());
}
