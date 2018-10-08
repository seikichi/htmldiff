mod builder;
mod html;
mod wu;

use builder::build_htmldiff;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{stdout, BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: htmldiff FILE1 FILE2");
        process::exit(1);
    }

    let html1 = read_html(&args[1]).expect("failed to read html file");
    let html2 = read_html(&args[2]).expect("failed to read html file");

    let stdout = stdout();
    let mut w = BufWriter::new(stdout.lock());
    build_htmldiff(&html1, &html2, |s: &str| {
        w.write(s.as_bytes()).expect("failed to write result");
    });
}

fn read_html(path: &str) -> io::Result<String> {
    let mut file = BufReader::new(File::open(path)?);
    let mut html = String::new();
    file.read_to_string(&mut html)?;
    Ok(html)
}
