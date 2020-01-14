mod message;
mod deserialize;

pub use message::ClientMessage;

const MESSAGE_END: char = '\n';
const PAYLOAD_START: char = ':';
const PAYLOAD_ITEM_SEPARATOR: char = ';';
const ESCAPE: char = '\\';
const MAX_MESSAGE_LENGTH: usize = 1024;

fn split(string: &str, separator: char, escape: char) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();

    let mut is_escaped = false;

    for c in string.chars() {
        if is_escaped {
            is_escaped = false;
        } else if c == escape {
            is_escaped = true;
        } else if c == separator {
            tokens.push(token);
            token = String::new();
            continue;
        }

        token.push(c);
    }

    tokens.push(token);

    tokens
}

fn find(string: &str, start: usize, to_find: char, escape: char) -> Option<usize> {
    assert!(start <= string.len(), "start must be <= string.len().");

    let mut is_escaped = false;
    for (i, c) in string.chars().skip(start).enumerate() {
        if is_escaped {
            is_escaped = false;

            if c == escape && to_find == escape {
                return Some(i);
            }

            continue;
        }

        if c == escape {
            is_escaped = true;
            continue;
        }

        if c == to_find {
            return Some(i);
        }
    }

    None
}


fn escape(string: &str, chars: &[char], escape: char) -> String
{
    let mut escaped = String::new();

    for sc in string.chars() {
        for &ec in chars {
            if sc == ec {
                escaped.push(escape);
                break;
            }
        }

        escaped.push(sc);
    }

    escaped
}


fn unescape(string: &str, chars: &[char], escape: char) -> String
{
    let mut unescaped = String::new();
    let mut is_escape = false;

    for sc in string.chars() {
        if is_escape {
            is_escape = false;

            let mut should_unescape = false;
            for &uc in chars {
                if sc == uc {
                    should_unescape = true;
                    break;
                }
            }
            if !should_unescape {
                unescaped.push(escape);
            }
        } else if sc == escape {
            is_escape = true;
            continue;
        }

        unescaped.push(sc);
    }

    unescaped
}


#[cfg(test)]
mod tests {
    use crate::proto::{escape, unescape};

    #[test]
    fn test_escape() {
        let escaped = escape(r"he;ll:o\j;:;el\\lo", &vec![';', ':'], '\\');
        assert_eq!(escaped, r"he\;ll\:o\j\;\:\;el\\lo");
    }

    #[test]
    fn test_unescape() {
        let unescaped = unescape(r"he;ll\:oj\\;\\\:\\\\;ello", &vec![';', ':'], '\\');
        assert_eq!(unescaped, r"he;ll:oj\\;\\:\\\\;ello");
    }


    #[test]
    fn test_escape_unescape() {
        let string = String::from(r"he;ll:o\j;:;el\\lo");
        let escaped = escape(&string, &vec![';', ':'], '\\');
        let unescaped = unescape(&escaped, &vec![';', ':'], '\\');
        assert_eq!(unescaped, string);
    }
}
