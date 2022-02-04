use crate::helper::is_c0_control;

pub struct URLPercentEncode;

pub enum PercentEncodeSet {
    UserInfo,
    Component,
    Path,
    Query,
    SpecialQuery,
    Fragment,
    C0Control,
}

impl URLPercentEncode {
    pub fn encode(
        input: &[u8],
        percent_encode_set: PercentEncodeSet,
        space_as_plus: bool,
    ) -> String {
        let mut output = String::new();

        for byte in input {
            if *byte == 0x0020 && space_as_plus {
                output.push('+');
            } else if !percent_encode_set.contains(*byte) {
                output.push(*byte as char);
            } else {
                output.push_str(&format!("%{:X?}", *byte as u32));
            }
        }

        output
    }
}

impl PercentEncodeSet {
    pub fn contains(&self, c: u8) -> bool {
        match self {
            Self::C0Control => is_c0_control(c as u32) || (c as u32) > 0x007E,
            Self::Fragment => {
                PercentEncodeSet::C0Control.contains(c)
                    || match c {
                        0x0020 | 0x0022 | 0x003C | 0x003E | 0x0060 => true,
                        _ => false,
                    }
            }
            Self::Query => {
                PercentEncodeSet::C0Control.contains(c)
                    || match c {
                        0x0020 | 0x0022 | 0x0023 | 0x003C | 0x003E => true,
                        _ => false,
                    }
            }
            Self::SpecialQuery => PercentEncodeSet::Query.contains(c) || c == 0x0027,
            Self::Path => {
                PercentEncodeSet::Query.contains(c)
                    || match c {
                        0x003F | 0x0060 | 0x007B | 0x007D => true,
                        _ => false,
                    }
            }
            Self::UserInfo => {
                PercentEncodeSet::Path.contains(c)
                    || match c {
                        0x002F | 0x003A | 0x003B | 0x003D | 0x0040 | 0x005B | 0x005E | 0x007C => {
                            true
                        }
                        _ => false,
                    }
            }
            Self::Component => {
                PercentEncodeSet::UserInfo.contains(c)
                    || match c {
                        0x0024 | 0x0026 | 0x002B | 0x002C => true,
                        _ => false,
                    }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_utf8_percent_encode() {
        assert_eq!(
            URLPercentEncode::encode("hello world".as_bytes(), PercentEncodeSet::UserInfo, false),
            "hello%20world"
        );
        assert_eq!(
            URLPercentEncode::encode(" ".as_bytes(), PercentEncodeSet::UserInfo, false),
            "%20"
        );
        assert_eq!(
            URLPercentEncode::encode("≡".as_bytes(), PercentEncodeSet::UserInfo, false),
            "%E2%89%A1"
        );
        assert_eq!(
            URLPercentEncode::encode("‽".as_bytes(), PercentEncodeSet::UserInfo, false),
            "%E2%80%BD"
        );
    }
}
