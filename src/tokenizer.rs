use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(String),
    StringLiteral(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    OpenParentheses,
    CloseParentheses,
    OpenCurlyBrackets,
    CloseCurlyBrackets,
    If,
    Else,
    While,
    And,
    Or,
    Not,
    True,
    False,
    Return,
    FunctionDef,
    Equals,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    MemberAcces,
    Separator,
    Empty,
}

pub fn tokenize(filedata: &str) -> Result<Vec<Token>, String> {
    let mut char_iter = filedata.chars().peekable();

    let mut tokens = vec![];

    // iterate over all characters in the code
    while let Some(character) = char_iter.next() {
        let token = match character {
            '_' | 'A'..='Z' | 'a'..='z' => tokenize_identifier(character, &mut char_iter),
            '0'..='9' => tokenize_number(character, &mut char_iter)?,
            '"' => tokenize_string(&mut char_iter)?,
            '(' => Token::OpenParentheses,
            ')' => Token::CloseParentheses,
            '{' => Token::OpenCurlyBrackets,
            '}' => Token::CloseCurlyBrackets,
            ',' => Token::Separator,
            '.' => Token::MemberAcces,
            '#' => {
                // comment until the end of the line
                for c in &mut char_iter {
                    if c == '\n' {
                        break;
                    }
                }
                Token::Empty
            }
            ' ' | '\t' | '\r' | '\n' => Token::Empty,
            _ => match char_iter.peek() {
                // check if there is a = character after the current character
                // for example +=
                Some(&'=') => {
                    char_iter.next();
                    match character {
                        '+' => Token::AddAssign,
                        '-' => Token::SubtractAssign,
                        '*' => Token::MultiplyAssign,
                        '/' => Token::DivideAssign,
                        '=' => Token::Equals,
                        '!' => Token::NotEqual,
                        '<' => Token::LessThanOrEqual,
                        '>' => Token::GreaterThanOrEqual,
                        _ => {
                            return Err(format!(
                                "Could not match character {:?} to any token",
                                character
                            ));
                        }
                    }
                }
                // if there is no = character after the current character
                // for example +
                _ => match character {
                    '+' => Token::Add,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '=' => Token::Assign,
                    '<' => Token::LessThan,
                    '>' => Token::GreaterThan,
                    '-' => Token::Subtract,
                    _ => {
                        return Err(format!(
                            "Could not match character {:?} to any token",
                            character
                        ));
                    }
                },
            },
        };

        // discard any redundant tokens
        if token != Token::Empty {
            tokens.push(token);
        }
    }

    Ok(tokens)
}

fn tokenize_identifier(first_char: char, char_iter: &mut Peekable<Chars>) -> Token {
    let mut data = first_char.to_string();
    while let Some('_') | Some('A'..='Z') | Some('a'..='z') | Some('0'..='9') = char_iter.peek() {
        data.push(char_iter.next().unwrap());
    }

    match data.as_str() {
        "if" => Token::If,
        "else" => Token::Else,
        "while" => Token::While,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "true" => Token::True,
        "false" => Token::False,
        "func" => Token::FunctionDef,
        "return" => Token::Return,
        _ => Token::Identifier(data),
    }
}

fn tokenize_number(first_char: char, char_iter: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut data = first_char.to_string();
    let mut has_decimals = false;

    while match char_iter.peek() {
        Some('_') | Some('0'..='9') => true,
        Some('A'..='Z') | Some('a'..='z') => {
            return Err(String::from("Found an alphabetical character in a number!"))
        }
        Some('.') => {
            if has_decimals {
                return Err(String::from("Multiple decimal separators in one number!"));
            }
            has_decimals = true;
            true
        }
        _ => false,
    } {
        data.push(char_iter.next().unwrap());
    }

    // erase underscores
    data.retain(|c| c != '_');
    // erase leading and trailing zeroes
    data = data.trim_matches('0').to_string();

    if data.is_empty() {
        data.push('0');
    }

    if data.ends_with('.') {
        data.push('0');
    }

    if data.starts_with('.') {
        data.insert(0, '0');
    }

    Ok(Token::Number(data))
}

fn tokenize_string(char_iter: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut data = String::new();
    while let Some(character) = char_iter.next() {
        match character {
            // we have reached the end of the string
            '"' => return Ok(Token::StringLiteral(data)),
            '\\' => {
                match char_iter.next() {
                    Some('\\') => data.push('\\'),
                    Some('n') => data.push('\n'),
                    Some('\'') => data.push('\''),
                    Some('"') => data.push('"'),
                    Some('U') => {
                        if let Some('+') = char_iter.next() {
                            let mut hex_input = String::new();
                            while let Some('0'..='9') | Some('a'..='f') | Some('A'..='F') =
                                char_iter.peek()
                            {
                                hex_input.push(char_iter.next().unwrap());
                            }
                            // this should always have valid input
                            let char_code = u32::from_str_radix(&hex_input, 16).unwrap();
                            if let Some(c) = std::char::from_u32(char_code) {
                                data.push(c);
                            } else {
                                return Err(format!(
                                    "Invalid unicode character code {}!",
                                    char_code
                                ));
                            }
                        } else {
                            return Err(String::from(
                                "Unicode literals need to be of the form \\U+xxxx",
                            ));
                        }
                    }

                    // skip newline
                    Some('\n') => (),
                    _ => {
                        return Err(String::from(
                            "Expected any of \\, n, \', \" or U after escape character \\!",
                        ));
                    }
                };
            }
            _ => data.push(character),
        }
    }
    Err(String::from("String literal not closed"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn comments() {
        assert_eq!(
            tokenize(
                "# this is a test to see if the 
        #tokenizer correctly igores # comments
        +"
            )
            .unwrap(),
            vec![Token::Add]
        );
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            tokenize("id = value").unwrap(),
            vec![
                Token::Identifier(String::from("id")),
                Token::Assign,
                Token::Identifier(String::from("value"))
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            tokenize("+-  / * =").unwrap(),
            vec![
                Token::Add,
                Token::Subtract,
                Token::Divide,
                Token::Multiply,
                Token::Assign
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            tokenize("12, 0000_25_._300, 0.0, 2., 0").unwrap(),
            vec![
                Token::Number(String::from("12")),
                Token::Separator,
                Token::Number(String::from("25.3")),
                Token::Separator,
                Token::Number(String::from("0.0")),
                Token::Separator,
                Token::Number(String::from("2.0")),
                Token::Separator,
                Token::Number(String::from("0")),
            ]
        );
    }

    #[test]
    fn errors() {
        assert!(tokenize("0000_.25_._300").is_err());
        assert!(tokenize("12units").is_err());
        assert!(tokenize("unknown symbol&").is_err());
        assert!(tokenize("\" this is an unclosed string # not a comment").is_err());
        assert!(tokenize(" \" unused \\ in a string literal \" ").is_err());
        assert!(tokenize("\"\\U+1021fFF\"").is_err());
    }

    #[test]
    fn member_access() {
        assert_eq!(
            tokenize("x.y").unwrap(),
            vec![
                Token::Identifier(String::from("x")),
                Token::MemberAcces,
                Token::Identifier(String::from("y"))
            ]
        );
    }

    #[test]
    fn functions() {
        assert_eq!(
            tokenize(
                "func add(x, y) {
            return x + y
            }"
            )
            .unwrap(),
            vec![
                Token::FunctionDef,
                Token::Identifier(String::from("add")),
                Token::OpenParentheses,
                Token::Identifier(String::from("x")),
                Token::Separator,
                Token::Identifier(String::from("y")),
                Token::CloseParentheses,
                Token::OpenCurlyBrackets,
                Token::Return,
                Token::Identifier(String::from("x")),
                Token::Add,
                Token::Identifier(String::from("y")),
                Token::CloseCurlyBrackets
            ]
        );
    }

    #[test]
    fn logic_operators() {
        assert_eq!(
            tokenize(
                "false or true and if not false 
            true 
        else 
            true or false"
            )
            .unwrap(),
            vec![
                Token::False,
                Token::Or,
                Token::True,
                Token::And,
                Token::If,
                Token::Not,
                Token::False,
                Token::True,
                Token::Else,
                Token::True,
                Token::Or,
                Token::False
            ]
        );
    }

    #[test]
    fn strings() {
        assert_eq!(
            tokenize(
                "
            \"this is a string \\\non one line\"
             \"\\\"\\\'\\n\\\\\" 
             \"\\U+4B\\U+3B6\\U+2764\\U+1F4af\"
             "
            )
            .unwrap(),
            vec![
                Token::StringLiteral(String::from("this is a string on one line")),
                Token::StringLiteral(String::from("\"\'\n\\")),
                Token::StringLiteral(String::from("Kζ❤💯")),
            ]
        );
    }
}
