use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{stdout, BufReader, BufWriter};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let html1 = {
        let mut file = BufReader::new(File::open(&args[0])?);
        let mut html = String::new();
        file.read_to_string(&mut html)?;
        html
    };

    let html2 = {
        let mut file = BufReader::new(File::open(&args[1])?);
        let mut html = String::new();
        file.read_to_string(&mut html)?;
        html
    };

    let old_words = convert_html_to_list_of_words(&html1);
    let new_words = convert_html_to_list_of_words(&html2);
    let ses = diff(&old_words, &new_words);

    let stdout = stdout();
    let mut w = BufWriter::new(stdout.lock());
    perform(&old_words, &new_words, &ses, |s: &str| {
        w.write(s.as_bytes()).unwrap();
    });

    Ok(())
}

fn is_tag(s: &str) -> bool {
    s.chars().next() == Some('<')
}

fn perform<'a, F>(old_words: &[&'a str], new_words: &[&'a str], ses: &[Edit], mut callback: F)
where
    F: FnMut(&str) -> (),
{
    for edit in ses {
        match edit {
            Edit::Common { old, new: _ } => {
                callback(old_words[*old]);
            }
            Edit::Add { new } => {
                let word = new_words[*new];
                if is_tag(word) {
                    callback(word);
                } else {
                    callback("<ins>");
                    callback(word);
                    callback("</ins>");
                }
            }
            Edit::Delete { old } => {
                let word = old_words[*old];
                if is_tag(word) {
                    callback(word);
                } else {
                    callback("<del>");
                    callback(word);
                    callback("</del>");
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Edit {
    Add { new: usize },
    Delete { old: usize },
    Common { new: usize, old: usize },
}

struct PointWithPrev {
    x: usize,
    y: usize,
    prev: isize,
}

struct Point {
    x: usize,
    y: usize,
}

// see: Sun Wu, Udi Manber, G.Myers, W.Miller, "An O(NP) Sequence Comparison Algorithm"
fn diff<T: Eq>(a: &[T], b: &[T]) -> Vec<Edit> {
    let m = a.len();
    let n = b.len();
    if m > n {
        return diff(b, a);
    }
    let offset = (m + 1) as isize;
    let delta = (n - m) as isize;
    let delta_offset = (delta + offset) as usize;

    // for SES
    let mut ids = vec![-1; m + n + 3];
    let mut points = vec![];
    {
        let mut snake = |k: isize, fp1: isize, fp2: isize| -> isize {
            let fp = std::cmp::max(fp1, fp2);
            let mut y = fp as usize;
            let mut x = (fp - k) as usize;
            while x < m && y < n && a[x] == b[y] {
                x += 1;
                y += 1;
            }
            // SES
            let ko = (k + offset) as usize;
            // NOTE: modify >= to > to change delete/insert orders.
            let prev = if fp1 >= fp2 { ids[ko - 1] } else { ids[ko + 1] };
            ids[ko] = points.len() as isize;
            points.push(PointWithPrev { x, y, prev });
            y as isize
        };

        let mut fp = vec![-1; m + n + 3];
        let mut p = -1;
        loop {
            p += 1;
            for k in -p..delta {
                let ko = (k + offset) as usize;
                fp[ko] = snake(k, fp[ko - 1] + 1, fp[ko + 1]);
            }
            for k in ((delta + 1)..=(delta + p)).rev() {
                let ko = (k + offset) as usize;
                fp[ko] = snake(k, fp[ko - 1] + 1, fp[ko + 1]);
            }
            fp[delta_offset] = snake(delta, fp[delta_offset - 1] + 1, fp[delta_offset + 1]);

            if fp[delta_offset] >= (n as isize) {
                break;
            }
        }
    }

    let mut route: Vec<Point> = vec![];
    let mut prev = ids[(delta + offset) as usize];
    while prev != -1 {
        let p = &points[prev as usize];
        route.push(Point { x: p.x, y: p.y });
        prev = p.prev;
    }
    let mut px = 0;
    let mut py = 0;
    let mut ses = vec![];
    for p in route.iter().rev() {
        while px < p.x || py < p.y {
            // compare (p.y - p.x) and (py - px)
            if p.y + px > p.x + py {
                ses.push(Edit::Add { new: py });
                py += 1;
            } else if p.y + px < p.x + py {
                ses.push(Edit::Delete { old: px });
                px += 1;
            } else {
                ses.push(Edit::Common { old: px, new: py });
                px += 1;
                py += 1;
            }
        }
    }
    ses
}

fn is_end_of_tag(c: char) -> bool {
    c == '>'
}

fn is_start_of_tag(c: char) -> bool {
    c == '<'
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

fn convert_html_to_list_of_words(s: &str) -> Vec<&str> {
    enum Mode {
        Char,
        Tag,
        Whitespace,
    }
    let mut words = vec![];
    let mut start = 0;
    let mut mode = Mode::Char;

    for (i, c) in s.char_indices() {
        match mode {
            Mode::Char if is_start_of_tag(c) => {
                if start != i {
                    unsafe {
                        words.push(s.get_unchecked(start..i));
                    }
                }
                start = i;
                mode = Mode::Tag;
            }
            Mode::Char if is_whitespace(c) => {
                if start != i {
                    unsafe {
                        words.push(s.get_unchecked(start..i));
                    }
                }
                start = i;
                mode = Mode::Whitespace;
            }
            Mode::Char => { /* continue */ }
            Mode::Tag if is_end_of_tag(c) => {
                unsafe {
                    words.push(s.get_unchecked(start..=i));
                }
                start = i + 1;
                mode = Mode::Char;
            }
            Mode::Tag => { /* continue */ }
            Mode::Whitespace if is_start_of_tag(c) => {
                if start != i {
                    unsafe {
                        words.push(s.get_unchecked(start..i));
                    }
                }
                start = i;
                mode = Mode::Tag;
            }
            Mode::Whitespace if is_whitespace(c) => { /* continue */ }
            Mode::Whitespace => {
                if start != i {
                    unsafe {
                        words.push(s.get_unchecked(start..i));
                    }
                }
                start = i;
                mode = Mode::Char;
            }
        }
    }

    if start < s.len() {
        words.push(&s[start..]);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_html() {
        let actual = convert_html_to_list_of_words("<p>Hello, world!</p>");
        let expected = vec!["<p>", "Hello,", " ", "world!", "</p>"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_diff() {
        let actual = diff(
            &vec!["み", "ん", "か", "ん", "じ", "ん"],
            &vec!["み", "か", "ん", "せ", "い", "じ", "ん"],
        );
        let expected = vec![
            Edit::Common { old: 0, new: 0 },
            Edit::Delete { old: 1 },
            Edit::Common { old: 2, new: 1 },
            Edit::Common { old: 3, new: 2 },
            Edit::Add { new: 3 },
            Edit::Add { new: 4 },
            Edit::Common { old: 4, new: 5 },
            Edit::Common { old: 5, new: 6 },
        ];
        assert_eq!(actual, expected);
    }
}
