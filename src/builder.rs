use std::cmp;

pub fn build_htmldiff<'a, F>(a: &'a str, b: &'a str, mut callback: F)
where
    F: FnMut(&'a str) -> (),
{
    let old_words = convert_html_to_list_of_words(a);
    let new_words = convert_html_to_list_of_words(b);
    let ses = diff(&old_words, &new_words);

    for edit in ses {
        match edit {
            Edit::Common { old, new: _ } => {
                callback(old_words[old]);
            }
            Edit::Add { new } => {
                let word = new_words[new];
                if is_tag(word) && !is_img_tag(word) {
                    callback(word);
                } else {
                    callback("<ins>");
                    callback(word);
                    callback("</ins>");
                }
            }
            Edit::Delete { old } => {
                let word = old_words[old];
                if is_tag(word) && !is_img_tag(word) {
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

fn diff<T: Eq>(a: &[T], b: &[T]) -> Vec<Edit> {
    SESBuilder::new(a, b).build()
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

struct SESBuilder<'a, T: 'a + Eq> {
    a: &'a [T],
    b: &'a [T],
    m: usize,
    n: usize,
    delta: isize,
    offset: isize,
    ids: Vec<isize>,
    points: Vec<PointWithPrev>,
    reverse: bool,
}

impl<'a, T: 'a + Eq> SESBuilder<'a, T> {
    fn new(a: &'a [T], b: &'a [T]) -> Self {
        let reverse = a.len() > b.len();
        let (a, b) = if reverse { (b, a) } else { (a, b) };
        let m = a.len();
        let n = b.len();
        let delta = (n - m) as isize;
        let offset = (m + 1) as isize;
        let ids = vec![-1; m + n + 3];
        let points = vec![];

        Self {
            a,
            b,
            m,
            n,
            delta,
            offset,
            ids,
            points,
            reverse,
        }
    }

    fn build(&mut self) -> Vec<Edit> {
        self.compare();
        self.build_ses()
    }

    // see: Sun Wu, Udi Manber, G.Myers, W.Miller, "An O(NP) Sequence Comparison Algorithm"
    fn compare(&mut self) {
        let delta = self.delta;
        let delta_offset = (self.delta + self.offset) as usize;

        let mut fp = vec![-1; self.m + self.n + 3];
        let mut p = -1;
        loop {
            p += 1;
            for k in -p..delta {
                let ko = (k + self.offset) as usize;
                fp[ko] = self.snake(k, fp[ko - 1] + 1, fp[ko + 1]);
            }
            for k in ((delta + 1)..=(delta + p)).rev() {
                let ko = (k + self.offset) as usize;
                fp[ko] = self.snake(k, fp[ko - 1] + 1, fp[ko + 1]);
            }
            fp[delta_offset] = self.snake(delta, fp[delta_offset - 1] + 1, fp[delta_offset + 1]);

            if fp[delta_offset] >= (self.n as isize) {
                break;
            }
        }
    }

    fn snake(&mut self, k: isize, fp1: isize, fp2: isize) -> isize {
        let fp = cmp::max(fp1, fp2);
        let mut y = fp as usize;
        let mut x = (fp - k) as usize;
        while x < self.m && y < self.n && self.a[x] == self.b[y] {
            x += 1;
            y += 1;
        }

        let ko = (k + self.offset) as usize;
        // NOTE: modify >= to > to change delete/insert orders.
        let prev = if fp1 >= fp2 {
            self.ids[ko - 1]
        } else {
            self.ids[ko + 1]
        };
        self.ids[ko] = self.points.len() as isize;
        self.points.push(PointWithPrev { x, y, prev });

        y as isize
    }

    fn build_ses(&mut self) -> Vec<Edit> {
        let mut route: Vec<Point> = vec![];
        let mut prev = self.ids[(self.delta + self.offset) as usize];
        while prev != -1 {
            let p = &self.points[prev as usize];
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
                    ses.push(if self.reverse {
                        Edit::Delete { old: py }
                    } else {
                        Edit::Add { new: py }
                    });
                    py += 1;
                } else if p.y + px < p.x + py {
                    ses.push(if self.reverse {
                        Edit::Add { new: px }
                    } else {
                        Edit::Delete { old: px }
                    });
                    px += 1;
                } else {
                    ses.push(if self.reverse {
                        Edit::Common { old: py, new: px }
                    } else {
                        Edit::Common { old: px, new: py }
                    });
                    px += 1;
                    py += 1;
                }
            }
        }
        ses
    }
}

fn is_img_tag(s: &str) -> bool {
    s.starts_with("<img")
}

fn is_tag(s: &str) -> bool {
    s.chars().next() == Some('<')
}

enum Mode {
    Char,
    Tag,
    Whitespace,
}

fn convert_html_to_list_of_words(s: &str) -> Vec<&str> {
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
            Mode::Char if is_in_word(c) => { /* continue */ }
            Mode::Char => {
                if start != i {
                    unsafe {
                        words.push(s.get_unchecked(start..i));
                    }
                }
                start = i;
            }
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

fn is_end_of_tag(c: char) -> bool {
    c == '>'
}

fn is_start_of_tag(c: char) -> bool {
    c == '<'
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

fn is_in_word(c: char) -> bool {
    c.is_alphanumeric() || c == '#' || c == '@'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_html() {
        let actual = convert_html_to_list_of_words("<p>Hello, world!</p>");
        let expected = vec!["<p>", "Hello", ",", " ", "world", "!", "</p>"];
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
