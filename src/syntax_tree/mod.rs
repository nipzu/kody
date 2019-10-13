use crate::runtime::add_function;
use crate::tokenizer::Token;

mod expression_parser;
use expression_parser::parse_expression_tokens;

#[derive(Debug, PartialEq)]
pub enum KodyNode {
    WhileStatement {
        condition: Box<KodyNode>,
        action: Box<KodyNode>,
    },
    IfStatement {
        condition: Box<KodyNode>,
        action: Box<KodyNode>,
        else_action: Option<Box<KodyNode>>,
    },
    CodeBlock {
        statements: Vec<KodyNode>,
    },
    ReturnFromFunction {
        return_value: Box<KodyNode>,
    },
    GetConstant {
        id: u64,
    },
    SetVariable {
        variable: Box<KodyNode>,
        value: Box<KodyNode>,
    },
    CallFunction {
        function: Box<KodyNode>,
        arguments: Vec<KodyNode>,
    },
    GetMember {
        base_object: Box<KodyNode>,
        member_name_id: u64,
    },
    GetVariable {
        name_id: u64,
    },
}

pub fn parse_tokens(tokens: &[Token]) -> Result<KodyNode, String> {
    parse_code_block(tokens)
}

fn parse_code_block(tokens: &[Token]) -> Result<KodyNode, String> {
    let expressions = identify_expressions(tokens)?;
    let mut statements = vec![];

    for expression in expressions {
        statements.push(parse_expression_tokens(expression)?);
    }

    Ok(KodyNode::CodeBlock { statements })
}

fn get_if_expression_tokens(tokens: &[Token]) -> Result<(&[Token], &[Token]), String> {
    if tokens[1..tokens.len()].is_empty() {
        return Err(String::from("Expected tokens after if"));
    }

    let (condition, other) = get_next_expression(&tokens[1..tokens.len()])?;

    if other.is_empty() {
        return Err(String::from(
            "Expected tokens after condition in if expression",
        ));
    }

    let (action, other) = get_next_expression(&other)?;

    if other.first() == Some(&Token::Else) {
        if other.len() < 2 {
            return Err(String::from("Expected tokens after else!"));
        }

        let (else_action, _) = get_next_expression(&other[1..other.len()])?;

        return Ok(tokens.split_at(condition.len() + action.len() + else_action.len() + 2));
    }

    Ok(tokens.split_at(condition.len() + action.len() + 1))
}

fn get_while_expression_tokens(tokens: &[Token]) -> Result<(&[Token], &[Token]), String> {
    if tokens[1..tokens.len()].is_empty() {
        return Err(String::from("Expected tokens after while!"));
    }
    let (condition, other) = get_next_expression(&tokens[1..tokens.len()])?;

    if other.is_empty() {
        return Err(String::from(
            "Expected tokens after condition in while statement",
        ));
    }

    let (action, _) = get_next_expression(&other)?;

    Ok(tokens.split_at(condition.len() + action.len() + 1))
}

fn get_function_tokens(tokens: &[Token]) -> Result<(&[Token], &[Token]), String> {
    let function_name;
    if let Some(Token::Identifier(name)) = tokens.get(1) {
        function_name = name;
    } else {
        return Err(String::from("Expected identifier after function keyword!"));
    }

    if let Some(Token::OpenParentheses) = tokens.get(2) {
    } else {
        return Err(String::from(
            "Expected parentheses after function identifier!",
        ));
    }

    let mut argument_iter = tokens.iter().skip(3);
    let mut arguments = vec![];
    let mut argument_len = 0;

    match argument_iter.next() {
        Some(Token::Identifier(name)) => {
            arguments.push(name.as_str());
            argument_len += 1
        }
        Some(Token::CloseParentheses) => (),
        _ => {
            return Err(String::from(
                "Unexpexted token after ( in function definition!",
            ))
        }
    }

    while match argument_iter.next() {
        Some(&Token::Separator) => true,
        Some(&Token::CloseParentheses) => false,
        _ => return Err(String::from("Unexpexted token in function arguments!")),
    } {
        argument_len += 1;
        if let Some(Token::Identifier(name)) = argument_iter.next() {
            arguments.push(name);
        } else {
            return Err(String::from("Unexpexted token in function arguments!"));
        }
    }

    let body_tokens = get_next_expression(&tokens[4 + argument_len * 2 - 1..tokens.len()])?.0;

    let body_len = body_tokens.len();

    let body = parse_expression_tokens(body_tokens)?;

    let total_len = argument_len * 2 - 1 + body_len + 4;

    add_function(function_name, arguments, Box::new(body));

    Ok((tokens.split_at(total_len).1, &[]))
}

fn get_next_expression(tokens: &[Token]) -> Result<(&[Token], &[Token]), String> {
    assert!(!tokens.is_empty());

    if tokens.first() == Some(&Token::If) {
        return get_if_expression_tokens(&tokens);
    }

    if tokens.first() == Some(&Token::While) {
        return get_while_expression_tokens(&tokens);
    }

    if tokens.first() == Some(&Token::FunctionDef) {
        return get_function_tokens(&tokens);
    }

    let mut token_iterator = tokens.iter().enumerate().peekable();

    if tokens.first() == Some(&Token::Return) {
        if tokens.len() == 1 {
            return Ok(tokens.split_at(1));
        }
        // skip the return token and then return the following expression
        token_iterator.next();
    }

    while let Some((i, token)) = token_iterator.next() {
        match token {
            Token::Identifier(_)
            | Token::Number(_)
            | Token::StringLiteral(_)
            | Token::CloseParentheses
            | Token::CloseCurlyBrackets
            | Token::True
            | Token::False => {
                if let Token::Identifier(_)
                | Token::Number(_)
                | Token::StringLiteral(_)
                | Token::OpenCurlyBrackets
                | Token::If
                | Token::While
                | Token::Else
                | Token::True
                | Token::False
                | Token::Return
                | Token::FunctionDef = token_iterator.peek().unwrap_or(&(0, &Token::Empty)).1
                {
                    return Ok(tokens.split_at(i + 1));
                }
            }
            Token::FunctionDef => {
                return Err(String::from(
                    "Unfinished expression before function definition!",
                ));
            }
            Token::Else => {
                return Err(String::from("Unexpexted else token"));
            }
            Token::Return => {
                return Err(String::from("Unexpexted return token"));
            }
            Token::If | Token::While => {
                for _ in 0..get_next_expression(&tokens[i..tokens.len()])?.0.len() - 2 {
                    token_iterator.next();
                }
            }
            Token::OpenCurlyBrackets => {
                let mut indent_level = 0;
                while match token_iterator.peek() {
                    Some((_, Token::OpenCurlyBrackets)) => {
                        indent_level += 1;
                        true
                    }
                    Some((_, Token::CloseCurlyBrackets)) => {
                        if indent_level == 0 {
                            false
                        } else {
                            indent_level -= 1;
                            true
                        }
                    }
                    None => return Err(String::from("Unclosed {")),
                    _ => true,
                } {
                    token_iterator.next();
                }
            }
            _ => (),
        }
    }

    Ok((tokens, &[]))
}

fn identify_expressions(tokens: &[Token]) -> Result<Vec<&[Token]>, String> {
    let mut expressions = vec![];
    let mut remaining_tokens = tokens;

    // TODO can be made simpler when if let chains gets stabilized
    #[allow(irrefutable_let_patterns)]
    while let (expression, other_tokens) = get_next_expression(remaining_tokens)? {
        expressions.push(expression);
        if other_tokens.is_empty() {
            break;
        }
        remaining_tokens = other_tokens;
    }

    Ok(expressions)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn negation_and_subtraction() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::Identifier(String::from("x")),
                Token::Assign,
                Token::Number(String::from("3")),
                Token::Divide,
                Token::Subtract,
                Token::Number(String::from("5"))
            ]),
            Ok(KodyNode::SetVariable {
                variable: Box::new(KodyNode::GetVariable { name_id: 0 }),
                value: Box::new(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        KodyNode::GetConstant { id: 0 },
                        KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![KodyNode::GetConstant { id: 0 }]
                        }
                    ]
                })
            })
        );
        assert_eq!(
            parse_expression_tokens(&[
                Token::Number(String::from("5")),
                Token::Subtract,
                Token::Number(String::from("3")),
            ]),
            Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![
                    KodyNode::GetConstant { id: 0 },
                    KodyNode::GetConstant { id: 0 }
                ]
            })
        );
        assert!(parse_expression_tokens(&[
            Token::Number(String::from("3")),
            Token::Add,
            Token::Subtract,
            Token::Number(String::from("2"))
        ])
        .is_err());
    }

    #[test]
    fn simple_expressions() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::Identifier(String::from("x")),
                Token::Assign,
                Token::Identifier(String::from("y")),
                Token::Add,
                Token::Number(String::from("1"))
            ]),
            Ok(KodyNode::SetVariable {
                variable: Box::new(KodyNode::GetVariable { name_id: 0 }),
                value: Box::new(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        KodyNode::GetVariable { name_id: 0 },
                        KodyNode::GetConstant { id: 0 },
                    ]
                })
            })
        );
        assert_eq!(
            parse_expression_tokens(&[
                Token::Identifier(String::from("print")),
                Token::OpenParentheses,
                Token::Identifier(String::from("y")),
                Token::Separator,
                Token::Number(String::from("1")),
                Token::Add,
                Token::Number(String::from("2")),
                Token::CloseParentheses,
            ]),
            Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetVariable { name_id: 0 }),
                arguments: vec![
                    KodyNode::GetVariable { name_id: 0 },
                    KodyNode::CallFunction {
                        function: Box::new(KodyNode::GetConstant { id: 0 }),
                        arguments: vec![
                            KodyNode::GetConstant { id: 0 },
                            KodyNode::GetConstant { id: 0 }
                        ]
                    },
                ]
            })
        );
    }

    #[test]
    fn parentheses() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::Identifier(String::from("a")),
                Token::Multiply,
                Token::OpenParentheses,
                Token::Number(String::from("2")),
                Token::Subtract,
                Token::Identifier(String::from("b")),
                Token::CloseParentheses
            ]),
            Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![
                    KodyNode::GetVariable { name_id: 0 },
                    KodyNode::CallFunction {
                        function: Box::new(KodyNode::GetConstant { id: 0 }),
                        arguments: vec![
                            KodyNode::GetConstant { id: 0 },
                            KodyNode::GetVariable { name_id: 0 }
                        ]
                    }
                ]
            })
        );
    }

    #[test]
    fn if_expression() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::Identifier(String::from("a")),
                Token::Add,
                Token::If,
                Token::True,
                Token::OpenCurlyBrackets,
                Token::Identifier(String::from("a")),
                Token::Assign,
                Token::Number(String::from("5")),
                Token::Identifier(String::from("a")),
                Token::CloseCurlyBrackets
            ]),
            Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![
                    KodyNode::GetVariable { name_id: 0 },
                    KodyNode::IfStatement {
                        condition: Box::new(KodyNode::GetConstant { id: 0 }),
                        action: Box::new(KodyNode::CodeBlock {
                            statements: vec![
                                KodyNode::SetVariable {
                                    variable: Box::new(KodyNode::GetVariable { name_id: 0 }),
                                    value: Box::new(KodyNode::GetConstant { id: 0 })
                                },
                                KodyNode::GetVariable { name_id: 0 }
                            ]
                        }),
                        else_action: None
                    }
                ]
            })
        )
    }

    #[test]
    fn logic_operators() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::Not,
                Token::OpenParentheses,
                Token::True,
                Token::And,
                Token::False,
                Token::Or,
                Token::True,
                Token::CloseParentheses
            ]),
            Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![
                                KodyNode::GetConstant { id: 0 },
                                KodyNode::GetConstant { id: 0 }
                            ]
                        },
                        KodyNode::GetConstant { id: 0 }
                    ]
                }]
            })
        )
    }

    #[test]
    fn control_flow() {
        assert_eq!(
            parse_expression_tokens(&[
                Token::If,
                Token::Identifier(String::from("y")),
                Token::Equals,
                Token::Number(String::from("1")),
                Token::Identifier(String::from("print")),
                Token::OpenParentheses,
                Token::Identifier(String::from("y")),
                Token::CloseParentheses
            ]),
            Ok(KodyNode::IfStatement {
                condition: Box::new(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        KodyNode::GetVariable { name_id: 0 },
                        KodyNode::GetConstant { id: 0 }
                    ]
                }),
                action: Box::new(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetVariable { name_id: 0 }),
                    arguments: vec![KodyNode::GetVariable { name_id: 0 }]
                }),
                else_action: None
            })
        );
        assert_eq!(
            parse_expression_tokens(&[
                Token::While,
                Token::Identifier(String::from("check")),
                Token::OpenParentheses,
                Token::Identifier(String::from("x")),
                Token::CloseParentheses,
                Token::OpenCurlyBrackets,
                Token::Identifier(String::from("x")),
                Token::DivideAssign,
                Token::Identifier(String::from("y")),
                Token::Identifier(String::from("y")),
                Token::Assign,
                Token::Number(String::from("2")),
                Token::CloseCurlyBrackets,
            ]),
            Ok(KodyNode::WhileStatement {
                condition: Box::new(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetVariable { name_id: 0 }),
                    arguments: vec![KodyNode::GetVariable { name_id: 0 },]
                }),
                action: Box::new(KodyNode::CodeBlock {
                    statements: vec![
                        KodyNode::SetVariable {
                            variable: Box::new(KodyNode::GetVariable { name_id: 0 }),
                            value: Box::new(KodyNode::CallFunction {
                                function: Box::new(KodyNode::GetConstant { id: 0 }),
                                arguments: vec![
                                    KodyNode::GetVariable { name_id: 0 },
                                    KodyNode::GetVariable { name_id: 0 }
                                ]
                            })
                        },
                        KodyNode::SetVariable {
                            variable: Box::new(KodyNode::GetVariable { name_id: 0 }),
                            value: Box::new(KodyNode::GetConstant { id: 0 })
                        }
                    ]
                })
            })
        );
    }
}