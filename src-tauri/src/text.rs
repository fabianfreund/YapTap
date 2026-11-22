//! The text pipeline between the model and the cursor. Pure functions only,
//! so every rule is unit-tested.

use crate::settings::Replacement;

pub struct Options<'a> {
    pub replacements: &'a [Replacement],
    pub trailing_space: bool,
}

pub fn process(raw: &str, opts: &Options) -> String {
    let mut text = collapse_spaces(raw.trim());
    text = spoken_breaks(&text);
    text = apply_replacements(&text, opts.replacements);
    if opts.trailing_space && !text.is_empty() && !text.ends_with('\n') {
        text.push(' ');
    }
    text
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// "new line" / "new paragraph" (and German "neue Zeile" / "neuer Absatz")
/// become line breaks. The model often adds punctuation around the phrase,
/// so that is removed too.
fn spoken_breaks(s: &str) -> String {
    const PHRASES: &[(&str, &str)] = &[
        ("new paragraph", "\n\n"),
        ("neuer absatz", "\n\n"),
        ("new line", "\n"),
        ("newline", "\n"),
        ("neue zeile", "\n"),
    ];
    let mut out = s.to_string();
    for (phrase, brk) in PHRASES {
        loop {
            let lower = out.to_lowercase();
            let Some(start) = find_word(&lower, phrase) else { break };
            let end = start + phrase.len();
            // Swallow punctuation and spaces the model put around the phrase.
            let before = out[..start].trim_end_matches([' ', ',']);
            let after = out[end..].trim_start_matches(['.', ',', '!', '?', ' ']);
            let after = capitalize_first(after);
            out = format!("{before}{brk}{after}");
        }
    }
    out.replace(" \n", "\n")
}

/// Byte index of `needle` in `hay` where it stands as whole words.
/// Both must be lowercase already.
fn find_word(hay: &str, needle: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(pos) = hay[from..].find(needle) {
        let start = from + pos;
        let end = start + needle.len();
        let ok_before = hay[..start].chars().last().is_none_or(|c| !c.is_alphanumeric());
        let ok_after = hay[end..].chars().next().is_none_or(|c| !c.is_alphanumeric());
        if ok_before && ok_after {
            return Some(start);
        }
        from = end;
    }
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Case-insensitive whole-word replacements, in list order.
fn apply_replacements(s: &str, list: &[Replacement]) -> String {
    let mut out = s.to_string();
    for r in list {
        let from = r.from.trim().to_lowercase();
        if from.is_empty() {
            continue;
        }
        let mut search_from = 0;
        loop {
            let lower = out.to_lowercase();
            // to_lowercase can change byte lengths for some scripts; only
            // replace when the positions still line up.
            if lower.len() != out.len() {
                break;
            }
            let Some(pos) = find_word(&lower[search_from..], &from) else { break };
            let start = search_from + pos;
            let end = start + from.len();
            out.replace_range(start..end, &r.to);
            search_from = start + r.to.len();
            if search_from > out.len() {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(reps: &[Replacement]) -> Options<'_> {
        Options {
            replacements: reps,
            trailing_space: false,
        }
    }

    fn rep(from: &str, to: &str) -> Replacement {
        Replacement {
            from: from.into(),
            to: to.into(),
        }
    }

    #[test]
    fn trims_and_collapses_spaces() {
        assert_eq!(process("  hello   world  ", &opts(&[])), "hello world");
    }

    #[test]
    fn trailing_space_is_added() {
        let o = Options {
            replacements: &[],
            trailing_space: true,
        };
        assert_eq!(process("Hi there.", &o), "Hi there. ");
        assert_eq!(process("", &o), "");
    }

    #[test]
    fn new_line_becomes_break() {
        assert_eq!(
            process("Hello, new line. how are you?", &opts(&[])),
            "Hello\nHow are you?"
        );
        assert_eq!(
            process("First point. New paragraph. Second point.", &opts(&[])),
            "First point.\n\nSecond point."
        );
        assert_eq!(process("Hallo neue Zeile wie geht's", &opts(&[])), "Hallo\nWie geht's");
    }

    #[test]
    fn phrase_inside_word_is_left_alone() {
        assert_eq!(process("renew lines", &opts(&[])), "renew lines");
    }

    #[test]
    fn replacements_are_whole_word_and_case_insensitive() {
        let reps = [rep("yap tap", "YapTap"), rep("cat", "dog")];
        assert_eq!(
            process("I love Yap Tap and my cat, not concat.", &opts(&reps)),
            "I love YapTap and my dog, not concat."
        );
    }

    #[test]
    fn replacement_can_contain_its_source() {
        let reps = [rep("tap", "tap tap")];
        assert_eq!(process("tap", &opts(&reps)), "tap tap");
    }
}
