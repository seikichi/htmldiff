mod builder;

use builder::build_htmldiff;

pub fn htmldiff(a: &str, b: &str) -> String {
    let mut diff = String::new();
    build_htmldiff(a, b, |s: &str| diff.push_str(s));
    diff
}
