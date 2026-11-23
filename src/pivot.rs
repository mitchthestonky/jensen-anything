//! The narrative rewrite. A single left-to-right pass over the prose, which
//! never touches a digit, a URL, inline code or a fenced block.

/// Ordinary business words and their AI infrastructure equivalents. No
/// replacement contains a digit, and each contains at least one AI term.
pub const PHRASES: &[(&str, &str)] = &[
    ("supply chain", "compute supply chain"),
    ("customer service", "agentic inference support"),
    ("customer support", "agentic inference support"),
    ("business model", "foundation model"),
    ("head office", "AI command cluster"),
    ("small business", "sovereign AI tenant"),
    ("small businesses", "sovereign AI tenants"),
    ("word of mouth", "model-to-model token exchange"),
    ("customers", "inference endpoints"),
    ("customer", "inference endpoint"),
    ("clients", "inference tenants"),
    ("client", "inference tenant"),
    ("employees", "agents"),
    ("employee", "agent"),
    ("staff", "agentic workforce"),
    ("workers", "agents"),
    ("worker", "agent"),
    ("team", "agent swarm"),
    ("teams", "agent swarms"),
    ("manager", "orchestration agent"),
    ("managers", "orchestration agents"),
    ("warehouse", "AI factory"),
    ("warehouses", "AI factories"),
    ("factory", "AI factory"),
    ("factories", "AI factories"),
    ("depot", "GPU depot"),
    ("depots", "GPU depots"),
    ("shop", "inference edge node"),
    ("shops", "inference edge nodes"),
    ("store", "inference edge node"),
    ("stores", "inference edge nodes"),
    ("branch", "edge inference cluster"),
    ("branches", "edge inference clusters"),
    ("location", "compute region"),
    ("locations", "compute regions"),
    ("van", "mobile GPU cluster"),
    ("vans", "mobile GPU clusters"),
    ("truck", "GPU cluster"),
    ("trucks", "GPU clusters"),
    ("fleet", "GPU fleet"),
    ("delivery", "token delivery"),
    ("deliveries", "token deliveries"),
    ("shipping", "token streaming"),
    ("product", "model"),
    ("products", "models"),
    ("service", "inference service"),
    ("services", "inference services"),
    ("software", "accelerated computing"),
    ("spreadsheet", "foundation model"),
    ("spreadsheets", "foundation models"),
    ("database", "vector database for inference"),
    ("data", "training data"),
    ("sales", "token throughput"),
    ("marketing", "AI prompt engineering"),
    ("orders", "inference requests"),
    ("order", "inference request"),
    ("repair", "model retraining"),
    ("repairs", "model retraining runs"),
    ("machine", "accelerated compute node"),
    ("machines", "accelerated compute nodes"),
    ("equipment", "accelerated compute"),
    ("inventory", "GPU inventory"),
    ("platform", "AI infrastructure platform"),
    ("business", "AI infrastructure business"),
    ("company", "AI infrastructure company"),
    ("industry", "AI industry"),
    ("market", "total addressable compute market"),
    ("growth", "compute scaling"),
    ("efficiency", "inference efficiency"),
    ("quality", "model quality"),
];

/// Words that count toward AI narrative density.
pub const AI_TERMS: &[&str] = &[
    "accelerated",
    "agent",
    "agentic",
    "agents",
    "ai",
    "cluster",
    "clusters",
    "compute",
    "gpu",
    "gpus",
    "inference",
    "model",
    "models",
    "neural",
    "token",
    "tokens",
    "training",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pivoted {
    pub text: String,
    pub replacements: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Density {
    pub words: u64,
    pub ai_terms: u64,
}

impl Density {
    pub fn of(text: &str) -> Self {
        let mut words = 0;
        let mut ai_terms = 0;
        for word in text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
        {
            words += 1;
            if AI_TERMS.contains(&word.to_ascii_lowercase().as_str()) {
                ai_terms += 1;
            }
        }
        Self { words, ai_terms }
    }

    /// AI terms per million words, rounded down.
    pub fn parts_per_million(self) -> u64 {
        self.ai_terms
            .saturating_mul(1_000_000)
            .checked_div(self.words)
            .unwrap_or(0)
    }
}

/// Every numeric token in order: `1,200.50`, `2024`, the `3` in `Q3`.
pub fn numbers(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() {
            let joins = matches!(bytes[index], b'.' | b',')
                && bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
            if bytes[index].is_ascii_digit() || joins {
                index += 1;
            } else {
                break;
            }
        }
        found.push(&text[start..index]);
    }
    found
}

pub fn pivot(text: &str) -> Pivoted {
    let protected = protected_mask(text);
    let mut phrases: Vec<&(&str, &str)> = PHRASES.iter().collect();
    phrases.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));

    let mut out = String::with_capacity(text.len() * 2);
    let mut replacements = 0;
    let mut index = 0;
    while index < text.len() {
        let current = text[index..].chars().next().unwrap_or_default();
        if !protected[index] && current.is_ascii_alphabetic() && at_word_start(text, index) {
            let hit = phrases
                .iter()
                .find(|(from, _)| matches_at(text, index, from, &protected));
            if let Some((from, to)) = hit {
                let source = &text[index..index + from.len()];
                let replacement = match_case(source, to);
                fix_article(&mut out, &replacement);
                out.push_str(&replacement);
                replacements += 1;
                index += from.len();
                continue;
            }
        }
        out.push(current);
        index += current.len_utf8();
    }
    Pivoted {
        text: out,
        replacements,
    }
}

fn at_word_start(text: &str, index: usize) -> bool {
    !text[..index]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
}

fn matches_at(text: &str, index: usize, phrase: &str, protected: &[bool]) -> bool {
    let end = index + phrase.len();
    let Some(candidate) = text.as_bytes().get(index..end) else {
        return false;
    };
    candidate.eq_ignore_ascii_case(phrase.as_bytes())
        && text.is_char_boundary(end)
        && !protected[index..end].iter().any(|&flag| flag)
        && !text[end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
}

fn match_case(source: &str, replacement: &str) -> String {
    let letters: Vec<char> = source.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if letters.first().is_some_and(|c| c.is_uppercase()) {
        let mut chars = replacement.chars();
        if let Some(first) = chars.next() {
            return first.to_uppercase().chain(chars).collect();
        }
    }
    replacement.to_string()
}

/// Keeps "a warehouse" from becoming "a AI factory".
fn fix_article(out: &mut String, replacement: &str) {
    let Some(before_space) = out.strip_suffix(' ') else {
        return;
    };
    let start = before_space
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_ascii_alphabetic())
        .map_or(0, |(at, c)| at + c.len_utf8());
    if start > 0 && !before_space[..start].ends_with(char::is_whitespace) {
        return;
    }
    let vowel = replacement
        .chars()
        .next()
        .is_some_and(|c| "aeiouAEIOU".contains(c));
    let fixed = match (&before_space[start..], vowel) {
        ("a", true) => "an",
        ("A", true) => "An",
        ("an", false) => "a",
        ("An", false) => "A",
        _ => return,
    };
    out.truncate(start);
    out.push_str(fixed);
    out.push(' ');
}

/// Marks bytes the pivot may not touch: fenced blocks, inline code, URLs,
/// link targets and HTML tags.
fn protected_mask(text: &str) -> Vec<bool> {
    let mut mask = vec![false; text.len()];
    let mut fence: Option<&str> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| trimmed.starts_with(m));
        let fenced = match (fence, marker) {
            (Some(open), Some(close)) if open == close => {
                fence = None;
                true
            }
            (Some(_), _) => true,
            (None, Some(open)) => {
                fence = Some(open);
                true
            }
            (None, None) => false,
        };
        if fenced {
            mask[offset..offset + line.len()].fill(true);
        } else {
            protect_inline(line, offset, &mut mask);
        }
        offset += line.len();
    }
    mask
}

fn protect_inline(line: &str, offset: usize, mask: &mut [bool]) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let rest = &line[index..];
        let span = if bytes[index] == b'`' {
            rest[1..].find('`').map(|end| end + 2)
        } else if rest.starts_with("](") {
            rest.find(')').map(|end| end + 1)
        } else if rest.starts_with("http://") || rest.starts_with("https://") {
            Some(
                rest.find(|c: char| c.is_whitespace() || c == ')')
                    .unwrap_or(rest.len()),
            )
        } else if bytes[index] == b'<'
            && bytes
                .get(index + 1)
                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'/')
        {
            rest.find('>').map(|end| end + 1)
        } else {
            None
        };
        match span {
            Some(length) => {
                mask[offset + index..offset + index + length].fill(true);
                index += length;
            }
            None => index += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacements_have_no_digits_and_add_an_ai_term() {
        for (from, to) in PHRASES {
            assert!(!to.contains(|c: char| c.is_ascii_digit()), "{to}");
            assert!(Density::of(to).ai_terms > 0, "{from} -> {to}");
        }
    }

    #[test]
    fn keeps_case_and_grammar() {
        assert_eq!(pivot("a warehouse").text, "an AI factory");
        assert_eq!(pivot("An employee").text, "An agent");
        assert_eq!(pivot("an order").text, "an inference request");
        assert_eq!(pivot("Customers").text, "Inference endpoints");
        assert_eq!(pivot("SALES").text, "TOKEN THROUGHPUT");
    }

    #[test]
    fn leaves_protected_text_alone() {
        let text = "`customer` [x](https://a.test/customer) <a href=\"store\">\n```\nstore\n```\n";
        assert_eq!(pivot(text).text, text);
    }

    #[test]
    fn does_not_match_inside_words() {
        assert_eq!(pivot("restore datum").text, "restore datum");
    }

    #[test]
    fn handles_multibyte_text() {
        assert_eq!(
            pivot("café customer — naïve").text,
            "café inference endpoint — naïve"
        );
    }

    #[test]
    fn extracts_numbers_in_order() {
        assert_eq!(
            numbers("Q3 made 1,200.50 in 2024."),
            ["3", "1,200.50", "2024"]
        );
    }
}
