#[derive(Debug, PartialEq)]
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
    OpenBrackets,
    CloseBrackets,
    Assignment,
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    EndOfLine,
    MemberAcces,
    Separator,
    Empty,
}

pub fn tokenize(filedata: &String) -> Result<Vec<Token>, String> {
    let mut characters = filedata.chars();

    let mut tokens = vec![];
    let mut next_char = characters.next();

    while let Some(character) = next_char {
        next_char = characters.next();
        let token = match character {
            '_' | 'A'...'Z' | 'a'...'z' => {
                let mut data = character.to_string();
                while match next_char.unwrap_or(' ') {
                    '_' | 'A'...'Z' | 'a'...'z' | '0'...'9' => true,
                    _ => false,
                } {
                    data.push(next_char.unwrap());
                    next_char = characters.next();
                }
                Token::Identifier(data)
            }

            '0'...'9' => {
                let mut data = character.to_string();
                let mut has_decimals = false;
                while match next_char.unwrap_or(' ') {
                    '_' | '0'...'9' => true,
                    'A'...'Z' | 'a'...'z' => return Err(String::from("Found an alphabetical character in a number!")),
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
                if data.chars().last() == Some('.') {
                    data.push('0');
                }
                if data.chars().next() == Some('.') {
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

            '[' => Token::OpenBrackets,

            ']' => Token::CloseBrackets,

            ';' => {
                if *tokens.last().unwrap_or(&Token::Empty) == Token::EndOfLine {
                    Token::Empty
                } else {
                    Token::EndOfLine
                }
            }

            '.' => Token::MemberAcces,

            ',' => Token::Separator,

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
                    Token::Assignment
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
                return Err(String::from(format!(
                    "Could not match character {} to any token",
                    character
                )));
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
                '\"' => data.push('\"'),
                'U' => {
                    return Err(String::from(
                        "Unicode code point construction not yet supported!",
                    ));
                }
                '\n' => (),
                '\0' => {
                    return Err(String::from(
                        "Expected newline after escape character \\, found end-of-file!",
                    ));
                }
                _ => {
                    while match characters.next().unwrap_or('\0') {
                        '\n' => false,
                        '\0' => {
                            return Err(String::from(
                                "Expected any of \\, n, \', \" or U after escape character \\!"
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
            tokenize(&String::from(
                "# this is a test to see if the 
        #tokenizer correctly igores # comments
        +"
            ))
            .unwrap(),
            vec![Token::Add]
        );
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            tokenize(&String::from("id = value")).unwrap(),
            vec![
                Token::Identifier(String::from("id")),
                Token::Assignment,
                Token::Identifier(String::from("value"))
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            tokenize(&String::from("+- -= / *= =")).unwrap(),
            vec![
                Token::Add,
                Token::Subtract,
                Token::SubtractAssign,
                Token::Divide,
                Token::MultiplyAssign,
                Token::Assignment
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            tokenize(&String::from("12, 0000_25_._300, 0.0, 2.")).unwrap(),
            vec![
                Token::Number(String::from("12")),
                Token::Separator,
                Token::Number(String::from("25.3")),
                Token::Separator,
                Token::Number(String::from("0.0")),
                Token::Separator,
                Token::Number(String::from("2.0")),
            ]
        );
    }

    #[test]
    fn errors() {
        assert!(tokenize(&String::from("0000_.25_._300")).is_err());
        assert!(tokenize(&String::from("12units")).is_err());
        assert!(tokenize(&String::from("unknown symbol&")).is_err());
        assert!(tokenize(&String::from("\" this is an unclosed string # not actually a comment")).is_err());
        assert!(tokenize(&String::from(" \" unused \\ in a string literal \" ")).is_err());
    }
}
