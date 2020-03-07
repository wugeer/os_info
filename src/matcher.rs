/// An implementation to match on simple strings.
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Considers the entire string (trimmed) to be the match.
    AllTrimmed,

    /// After finding the `prefix` followed by one or more spaces, returns the following word.
    #[allow(dead_code)]
    PrefixedWord { prefix: &'static str },

    /// Similar to `PrefixedWord`, but only if the word is a valid version.
    PrefixedVersion { prefix: &'static str },

    /// Key/Value file - principally seen on Oracle Linux with the /etc/os-release file.
    KeyValue,
}

impl Matcher {
    /// Find the match on the input `string`.
    pub fn find(&self, string: &str) -> Option<String> {
        match *self {
            Self::AllTrimmed => Some(string.trim().to_string()),
            Self::PrefixedWord { prefix } => {
                find_prefixed_word(string, prefix).map(|v| v.to_owned())
            }
            Self::PrefixedVersion { prefix } => find_prefixed_word(string, prefix)
                .filter(|&v| is_valid_version(v))
                .map(|v| v.to_owned()),
            Self::KeyValue => find_by_key(string, "ORACLE_SUPPORT_PRODUCT_VERSION"),
        }
    }
}

fn find_by_key(string: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = string.split('\n').collect();

    for line in lines {
        let kv: Vec<&str> = line.split('=').collect();
        if kv[0] == key {
            return Some(kv[1].to_owned());
        }
    }

    None
}

fn find_prefixed_word<'a>(string: &'a str, prefix: &str) -> Option<&'a str> {
    if let Some(prefix_start) = string.find(prefix) {
        // Ignore prefix and leading whitespace
        let string = &string[prefix_start + prefix.len()..].trim_start();

        // Find where the word boundary ends
        let word_end = string
            .find(|c: char| c.is_whitespace())
            .unwrap_or_else(|| string.len());
        let string = &string[..word_end];

        Some(string)
    } else {
        None
    }
}

fn is_valid_version(word: &str) -> bool {
    !word.starts_with('.') && !word.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn trimmed() {
        let data = [
            ("", Some("")),
            ("test", Some("test")),
            (" 		 test", Some("test")),
            ("test  	   ", Some("test")),
            ("  test 	", Some("test")),
        ];

        let matcher = Matcher::AllTrimmed;

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }

    #[test]
    fn prefixed_word() {
        let data = [
            ("", None),
            ("test", Some("")),
            ("test 1", Some("1")),
            (" test 1", Some("1")),
            ("test 1.2.3", Some("1.2.3")),
            (" 		test 1.2.3", Some("1.2.3")),
        ];

        let matcher = Matcher::PrefixedWord { prefix: "test" };

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }

    #[test]
    fn prefixed_version() {
        let data = [
            ("", None),
            ("test", Some("")),
            ("test 1", Some("1")),
            ("test .1", None),
            ("test 1.", None),
            ("test .1.", None),
            (" test 1", Some("1")),
            ("test 1.2.3", Some("1.2.3")),
            (" 		test 1.2.3", Some("1.2.3")),
        ];

        let matcher = Matcher::PrefixedVersion { prefix: "test" };

        for (input, expected) in &data {
            let result = matcher.find(input);
            assert_eq!(result.as_deref(), *expected);
        }
    }
}
