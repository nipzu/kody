#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(String),
    StringLiteral(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    OpenParentheses,
    CloseParentheses,
    OpenBraces,
    CloseBraces,
    If,
    Else,
    While,
    And,
    Or,
    Not,
    True,
    False,
    Return,
    Assign,
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


// TODO make better (skip_while)
pub fn tokenize(filedata: &str) -> Result<Vec<Token>, String> {
    let mut characters = filedata.chars();

    let mut tokens = vec![];
    let mut next_char = characters.next();

    while let Some(character) = next_char {
        next_char = characters.next();
        let token = match character {
            '_' | 'A'..='Z' | 'a'..='z' => {
                let mut data = character.to_string();
                while match next_char.unwrap_or(' ') {
                    '_' | 'A'..='Z' | 'a'..='z' | '0'..='9' => true,
                    _ => false,
                } {
                    data.push(next_char.unwrap());
                    next_char = characters.next();
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

            '0'..='9' => {
                let mut data = character.to_string();
                let mut has_decimals = false;
                while match next_char.unwrap_or(' ') {
                    '_' | '0'..='9' => true,
                    'A'..='Z' | 'a'..='z' => {
                        return Err(String::from("Found an alphabetical character in a number!"))
                    }
                    '.' => {
                        if has_decimals {
                            return Err(String::from("Multiple decimal separators in one number!"));
                        }
                        has_decimals = true;
                        true
                    }
                    _ => false,
                } {
                    data.push(next_char.unwrap());
                    next_char = characters.next();
                }

                data.retain(|c| c != '_');
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

                Token::Number(data)
            }

            '"' => {
                let string_data = tokenize_string(next_char.unwrap_or('\0'), &mut characters)?;
                next_char = characters.next();
                string_data
            }

            '+' => {
                if next_char == Some('=') {
                    next_char = characters.next();
                    Token::AddAssign
                } else {
                    Token::Add
                }
            }

            '-' => {
                if next_char == Some('=') {
                    next_char = characters.next();
                    Token::SubtractAssign
                } else {
                    Token::Subtract
                }
            }

            '*' => {
                if next_char == Some('=') {
                    next_char = characters.next();
                    Token::MultiplyAssign
                } else {
                    Token::Multiply
                }
            }

            '/' => {
                if next_char == Some('=') {
                    next_char = characters.next();
                    Token::DivideAssign
                } else {
                    Token::Divide
                }
            }

            '(' => Token::OpenParentheses,

            ')' => Token::CloseParentheses,

            '{' => Token::OpenBraces,

            '}' => Token::CloseBraces,

            ',' => Token::Separator,

            '.' => Token::MemberAcces,

            ' ' | '\t' | '\r' | '\n' => Token::Empty,

            '#' => {
                while match next_char.unwrap_or('\n') {
                    '\n' => false,
                    _ => true,
                } {
                    next_char = characters.next();
                }
                next_char = characters.next();
                Token::Empty
            }

            '=' => {
                if next_char.unwrap_or(' ') == '=' {
                    next_char = characters.next();
                    Token::Equals
                } else {
                    Token::Assign
                }
            }

            '!' => {
                if next_char.unwrap_or(' ') == '=' {
                    next_char = characters.next();
                    Token::NotEqual
                } else {
                    return Err(String::from("Unknown token \'!\'"));
                }
            }

            '<' => {
                if next_char.unwrap_or(' ') == '=' {
                    next_char = characters.next();
                    Token::LessThanOrEqual
                } else {
                    Token::LessThan
                }
            }

            '>' => {
                if next_char.unwrap_or(' ') == '=' {
                    next_char = characters.next();
                    Token::GreaterThanOrEqual
                } else {
                    Token::GreaterThan
                }
            }

            _ => {
                return Err(format!(
                    "Could not match character {} to any token",
                    character
                ));
            }
        };
        if token != Token::Empty {
            tokens.push(token);
        }
    }
    Ok(tokens)
}

fn tokenize_string(first_char: char, characters: &mut std::str::Chars) -> Result<Token, String> {
    let mut data = String::new();
    let mut next_char = Some(first_char);
    while match next_char.unwrap_or('\0') {
        '\0' => return Err(String::from("String literal not closed")),
        '"' => false,
        '\\' => {
            match characters.next().unwrap_or('\0') {
                '\\' => data.push('\\'),
                'n' => data.push('\n'),
                '\'' => data.push('\''),
                '"' => data.push('"'),
                'U' => {
                    // TODO implement unicode support
                    return Err(String::from(
                        "Unicode code point construction not yet supported!",
                    ));
                }
                '\n' => (),
                '\0' => {
                    return Err(String::from(
                        "Found end-of-file after escape character \\!",
                    ));
                }
                _ => {
                    // skip newline
                    while match characters.next().unwrap_or('\0') {
                        '\n' => false,
                        '\0' => {
                            return Err(String::from(
                                "Expected any of \\, n, \', \" or U after escape character \\!",
                            ));
                        }
                        _ => true,
                    } {}
                    next_char = characters.next();
                }
            };
            true
        }
        _ => {
            data.push(next_char.unwrap());
            next_char = characters.next();
            true
        }
    } {}
    Ok(Token::StringLiteral(data))
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
                Token::OpenBraces,
                Token::Return,
                Token::Identifier(String::from("x")),
                Token::Add,
                Token::Identifier(String::from("y")),
                Token::CloseBraces
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
}
