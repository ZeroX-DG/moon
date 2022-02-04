use crate::{helper::{contains_forbidden_host_code_point, is_url_c}, encode::{URLPercentEncode, PercentEncodeSet}};

pub struct HostParser;

fn report_validation_error() {
    // TODO
}

impl HostParser {
    pub fn parse(input: &str, is_not_special: bool) -> Option<String> {
        if input.starts_with("[") {
            if !input.ends_with("]") {
                report_validation_error();
                return None;
            }

            unimplemented!("IPv6 parsing is not supported yet");
        }

        if is_not_special {
            return HostParser::parse_opaque_host(input);
        }

        return Some(input.to_string());
    }

    fn parse_opaque_host(input: &str) -> Option<String> {
        if contains_forbidden_host_code_point(input) {
            report_validation_error();
            return None;
        }

        if input.contains(|c| !is_url_c(c) && c != '%') {
            report_validation_error();
        }

        // TODO: If input contains a U+0025 (%) and the two code points following it are not ASCII hex digits, validation error.

        return Some(URLPercentEncode::encode(input.as_bytes(), PercentEncodeSet::C0Control, false));
    }
}
