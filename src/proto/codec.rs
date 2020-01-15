use std::iter::Iterator;
use std::collections::LinkedList;
use crate::proto::deserialize::{DeserializeError, DeserializeErrorKind};

/// A character denoting message end.
pub const MESSAGE_END: char = '\n';

/// A character denoting that a payload is following after the header.
pub const PAYLOAD_START: char = ':';

/// A character denoting separation of two payload items
pub const PAYLOAD_ITEM_SEPARATOR: char = ';';

/// An escape character.
pub const ESCAPE: char = '\\';

/// Max length of the message after which the message is considered invalid.
pub const MAX_MESSAGE_LENGTH: usize = 1024;

/// Split the string by the separator that is not escape by the escape character.
pub fn split(string: &str, separator: char, escape: char) -> Vec<String> {
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

/// Find position in the string of the first byte of the character
/// which is not escaped by the escape character.
pub fn find(string: &str, to_find: char, escape: char) -> Option<usize> {
    let mut is_escaped = false;
    for (i, c) in string.char_indices() {
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

/// Escape characters in a string with the escape character.
pub fn escape(string: &str, chars: &[char], escape: char) -> String
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

/// Unescape characters in a string escaped by the escape character.
pub fn unescape(string: &str, chars: &[char], escape: char) -> String
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

/// A collection of a message payload items
/// that can be appended to back of the payload
/// or taken from the front of the payload.
pub struct Payload {
    items: LinkedList<String>
}

impl Payload {
    /// Create an empty payload - it has no items.
    pub fn empty() -> Self {
        Payload {
            items: LinkedList::new()
        }
    }

    /// Deserialize payload items from string.
    /// Even empty string is a non-empty payload consisting of one empty string item.
    pub fn deserialize(serialized: &str) -> Self {
        let parts = split(serialized, PAYLOAD_ITEM_SEPARATOR, ESCAPE);

        let items = parts.iter()
            .map(|part| unescape(part, &[ESCAPE, PAYLOAD_ITEM_SEPARATOR], ESCAPE))
            .collect();

        Payload {
            items
        }
    }

    /// Serialize payload items into a string.
    /// If the payload is empty None is returned.
    pub fn serialize(&self) -> Option<String> {
        if self.items.is_empty() {
            return None;
        }

        let escaped = self.items.iter()
            .map(|item| escape(&item, &[ESCAPE, PAYLOAD_ITEM_SEPARATOR], ESCAPE))
            .collect::<Vec<_>>();

        let mut serialized = String::new();

        let mut iterator = escaped.iter().peekable();
        loop {
            match iterator.next() {
                Some(item) => {
                    serialized.push_str(item);
                }
                None => {
                    break;
                }
            }

            if let Some(_) = iterator.peek() {
                serialized.push(PAYLOAD_ITEM_SEPARATOR);
            }
        }

        Some(serialized)
    }

    /// Put a string item into the payload.
    pub fn put_string(&mut self, string: String) {
        self.items.push_back(string);
    }

    /// Put an int item, which is serialized into a string.
    pub fn put_int(&mut self, int: i32) {
        self.items.push_back(int.to_string());
    }

    /// Take next item from the front of the payload.
    fn take_item(&mut self) -> Result<String, DeserializeError> {
        if let Some(item) = self.items.pop_front() {
            Ok(item)
        } else {
            Err(DeserializeError::new(DeserializeErrorKind::NoMorePayloadItems))
        }
    }

    /// Get a next string item.
    pub fn take_string(&mut self) -> Result<String, DeserializeError> {
        self.take_item()
    }

    /// Get an u8 integer item, which is deserialized from string.
    /// The item is taken from the payload even if the deserialization fails.
    pub fn take_u8(&mut self) -> Result<u8, DeserializeError> {
        let item = self.take_item()?;
        let int = item.parse()?;
        Ok(int)
    }
}


#[cfg(test)]
mod tests {
    use crate::proto::codec::{escape, unescape};

    #[test]
    fn test_escape() {
        let escaped = escape(r"he;ll:o\j;:;el\\lo", &[';', ':'], '\\');
        assert_eq!(escaped, r"he\;ll\:o\j\;\:\;el\\lo");
    }

    #[test]
    fn test_unescape() {
        let unescaped = unescape(r"he;ll\:oj\\;\\\:\\\\;ello", &[';', ':'], '\\');
        assert_eq!(unescaped, r"he;ll:oj\\;\\:\\\\;ello");
    }


    #[test]
    fn test_escape_unescape() {
        let string = String::from(r"he;ll:o\j;:;el\\lo");
        let escaped = escape(&string, &[';', ':'], '\\');
        let unescaped = unescape(&escaped, &[';', ':'], '\\');
        assert_eq!(unescaped, string);
    }
}