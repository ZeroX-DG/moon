pub mod structs;

use super::parser::structs::ComponentValue;
use super::parser::Parser;
use super::tokenizer::token::Token;
use super::tokenizer::Tokenizer;
use io::data_stream::DataStream;
use structs::*;

macro_rules! token_value {
    ($token:pat) => {
        ComponentValue::PerservedToken($token)
    };
}

pub fn parse_selector_str(selector: &str) -> Option<Selector> {
    let tokenizer = Tokenizer::new(selector.to_string());
    let mut parser = Parser::<Token>::new(tokenizer.run());
    let values = parser.parse_a_list_of_component_values();
    let mut data_stream = DataStream::new(values);
    parse_selector(&mut data_stream)
}

pub fn parse_selectors(values: &Vec<ComponentValue>) -> Vec<Selector> {
    let mut selectors = Vec::new();

    let mut data_stream = DataStream::new(values.clone());

    loop {
        if let Some(selector) = parse_selector(&mut data_stream) {
            selectors.push(selector);
            loop {
                // consume all white space
                if let Some(token_value!(Token::Whitespace)) = data_stream.peek() {
                    data_stream.next();
                } else {
                    break;
                }
            }
            if let Some(token_value!(Token::Comma)) = data_stream.next() {
                // there is a comma, let the parsing continue
                continue;
            }
            // no comma? the sequence ends here
            break;
        } else {
            data_stream.next();
        }

        if data_stream.is_eos() {
            break;
        }
    }
    return selectors;
}

pub fn parse_selector(data_stream: &mut DataStream<ComponentValue>) -> Option<Selector> {
    let mut selector_seqs: SelectorData = Vec::new();
    loop {
        if let Some(selector_seq) = parse_simple_selector_seq(data_stream) {
            if let Some(combinator) = parse_combinator(data_stream) {
                selector_seqs.push((selector_seq, Some(combinator)));
                continue;
            }
            selector_seqs.push((selector_seq, None));
            break;
        } else {
            data_stream.next();
        }

        if data_stream.is_eos() {
            break;
        }
    }

    match selector_seqs.len() {
        0 => None,
        _ => Some(Selector::new(selector_seqs)),
    }
}

pub fn parse_combinator(data_stream: &mut DataStream<ComponentValue>) -> Option<Combinator> {
    let next_values = data_stream.peek_next(4);

    if next_values.len() == 4 {
        return match (
            next_values[0],
            next_values[1],
            next_values[2],
            next_values[3],
        ) {
            // With space between combinator
            (
                token_value!(Token::Whitespace),
                token_value!(Token::Delim('+')),
                token_value!(Token::Whitespace),
                _,
            ) => {
                data_stream.next();
                data_stream.next();
                data_stream.next();
                Some(Combinator::NextSibling)
            }
            (
                token_value!(Token::Whitespace),
                token_value!(Token::Delim('~')),
                token_value!(Token::Whitespace),
                _,
            ) => {
                data_stream.next();
                data_stream.next();
                data_stream.next();
                Some(Combinator::SubsequentSibling)
            }
            (
                token_value!(Token::Whitespace),
                token_value!(Token::Delim('>')),
                token_value!(Token::Whitespace),
                _,
            ) => {
                data_stream.next();
                data_stream.next();
                data_stream.next();
                Some(Combinator::Child)
            }
            _ => None,
        };
    }

    let next_values = data_stream.peek_next(2);

    if next_values.len() == 2 {
        return match (next_values[0], next_values[1]) {
            // No space between combinator
            (token_value!(Token::Whitespace), _) => {
                data_stream.next();
                Some(Combinator::Descendant)
            }
            (token_value!(Token::Delim('+')), _) => {
                data_stream.next();
                Some(Combinator::NextSibling)
            }
            (token_value!(Token::Delim('~')), _) => {
                data_stream.next();
                Some(Combinator::SubsequentSibling)
            }
            (token_value!(Token::Delim('>')), _) => {
                data_stream.next();
                Some(Combinator::Child)
            }
            _ => None,
        };
    }

    None
}

pub fn parse_simple_selector_seq(
    data_stream: &mut DataStream<ComponentValue>,
) -> Option<SimpleSelectorSequence> {
    let mut seq = Vec::new();
    loop {
        if let Some(simple_selector) = parse_simple_selector(data_stream) {
            seq.push(simple_selector);
        } else {
            if let Some(value) = data_stream.peek() {
                match value {
                    ComponentValue::PerservedToken(Token::Whitespace) => break,
                    ComponentValue::PerservedToken(Token::Comma) => break,
                    _ => {
                        data_stream.next();
                    }
                }
            }
        }
        if data_stream.is_eos() {
            break;
        }
    }

    match seq.len() {
        0 => None,
        _ => Some(SimpleSelectorSequence::new(seq)),
    }
}

pub fn parse_simple_selector(
    data_stream: &mut DataStream<ComponentValue>,
) -> Option<SimpleSelector> {
    let next_token = data_stream.peek_clone();
    match next_token {
        Some(token_value!(Token::Ident(data))) => {
            data_stream.next();
            Some(SimpleSelector::new(SimpleSelectorType::Type, Some(data)))
        }
        Some(token_value!(Token::Delim('*'))) => {
            data_stream.next();
            Some(SimpleSelector::new(SimpleSelectorType::Universal, None))
        }
        Some(token_value!(Token::Hash(data, _))) => {
            data_stream.next();
            Some(SimpleSelector::new(SimpleSelectorType::ID, Some(data)))
        }
        Some(token_value!(Token::Delim('.'))) => {
            let next_values = data_stream.peek_next(2);
            if next_values.len() != 2 {
                return None;
            }
            if let token_value!(Token::Ident(data)) = next_values[1].clone() {
                data_stream.next();
                data_stream.next();
                return Some(SimpleSelector::new(SimpleSelectorType::Class, Some(data)));
            }
            None
        }
        // TODO: Support other selectors too
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::structs::Rule;
    use crate::parser::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn parse_simple_valid() {
        let css = "div.class#id { color: red; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 1);

            let expected = Selector::new(vec![(
                SimpleSelectorSequence::new(vec![
                    SimpleSelector::new(SimpleSelectorType::Type, Some("div".to_string())),
                    SimpleSelector::new(SimpleSelectorType::Class, Some("class".to_string())),
                    SimpleSelector::new(SimpleSelectorType::ID, Some("id".to_string())),
                ]),
                None,
            )]);

            assert_eq!(selectors.get(0), Some(&expected));
        }
    }

    #[test]
    fn parse_simple_valid_with_combinator() {
        let css = "div.class #id { color: red; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 1);

            let expected = Selector::new(vec![
                (
                    SimpleSelectorSequence::new(vec![
                        SimpleSelector::new(SimpleSelectorType::Type, Some("div".to_string())),
                        SimpleSelector::new(SimpleSelectorType::Class, Some("class".to_string())),
                    ]),
                    Some(Combinator::Descendant),
                ),
                (
                    SimpleSelectorSequence::new(vec![SimpleSelector::new(
                        SimpleSelectorType::ID,
                        Some("id".to_string()),
                    )]),
                    None,
                ),
            ]);

            assert_eq!(selectors.get(0), Some(&expected));
        }
    }

    #[test]
    fn parse_simple_invalid_with_combinator() {
        let css = "div.class > > #id { color: red; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 1);

            let expected = Selector::new(vec![
                (
                    SimpleSelectorSequence::new(vec![
                        SimpleSelector::new(SimpleSelectorType::Type, Some("div".to_string())),
                        SimpleSelector::new(SimpleSelectorType::Class, Some("class".to_string())),
                    ]),
                    Some(Combinator::Child),
                ),
                (
                    SimpleSelectorSequence::new(vec![SimpleSelector::new(
                        SimpleSelectorType::ID,
                        Some("id".to_string()),
                    )]),
                    None,
                ),
            ]);

            assert_eq!(selectors.get(0), Some(&expected));
        }
    }

    #[test]
    fn parse_nested() {
        let css = "div.class > #id > #name + div { color: red; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 1);

            let expected = Selector::new(vec![
                (
                    SimpleSelectorSequence::new(vec![
                        SimpleSelector::new(SimpleSelectorType::Type, Some("div".to_string())),
                        SimpleSelector::new(SimpleSelectorType::Class, Some("class".to_string())),
                    ]),
                    Some(Combinator::Child),
                ),
                (
                    SimpleSelectorSequence::new(vec![SimpleSelector::new(
                        SimpleSelectorType::ID,
                        Some("id".to_string()),
                    )]),
                    Some(Combinator::Child),
                ),
                (
                    SimpleSelectorSequence::new(vec![SimpleSelector::new(
                        SimpleSelectorType::ID,
                        Some("name".to_string()),
                    )]),
                    Some(Combinator::NextSibling),
                ),
                (
                    SimpleSelectorSequence::new(vec![SimpleSelector::new(
                        SimpleSelectorType::Type,
                        Some("div".to_string()),
                    )]),
                    None,
                ),
            ]);

            assert_eq!(selectors.get(0), Some(&expected));
        }
    }

    #[test]
    fn parse_multiple() {
        let css = "div.class, #name { color: black; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 2);

            let expected = Selector::new(vec![(
                SimpleSelectorSequence::new(vec![
                    SimpleSelector::new(SimpleSelectorType::Type, Some("div".to_string())),
                    SimpleSelector::new(SimpleSelectorType::Class, Some("class".to_string())),
                ]),
                None,
            )]);

            let expected2 = Selector::new(vec![(
                SimpleSelectorSequence::new(vec![SimpleSelector::new(
                    SimpleSelectorType::ID,
                    Some("name".to_string()),
                )]),
                None,
            )]);

            assert_eq!(selectors.get(0), Some(&expected));
            assert_eq!(selectors.get(1), Some(&expected2));
        }
    }

    #[test]
    fn parse_invalid() {
        let css = " { color: black; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let rules = parser.parse_a_stylesheet();
        let rule = rules.get(0).unwrap();

        if let Rule::QualifiedRule(rule) = rule {
            let selectors = parse_selectors(&rule.prelude);

            assert_eq!(selectors.len(), 0);
        }
    }

    #[test]
    fn test_specificity() {
        let css = "#div.class#name";
        let selector = parse_selector_str(css);
        let specificity = selector.expect("Failed to parse selector").specificity();

        assert_eq!(specificity, Specificity::new(2, 1, 0));
    }
}
