use html;
use wu::{diff, Edit};

pub fn build_htmldiff<'a, F>(a: &'a str, b: &'a str, mut callback: F)
where
    F: FnMut(&'a str) -> (),
{
    let old_words = html::split(a);
    let new_words = html::split(b);
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

fn is_img_tag(s: &str) -> bool {
    s.starts_with("<img")
}

fn is_tag(s: &str) -> bool {
    s.starts_with("<")
}
