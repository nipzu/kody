use crate::runtime::add_function;
use crate::tokenizer::Token;

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

// FIXME, TODO this thing needs some heavy reformatting
fn parse_expression_tokens(tokens: &[Token]) -> Result<KodyNode, String> {
    assert!(!tokens.is_empty());

    // parentheses
    if tokens.first() == Some(&Token::OpenParentheses)
        && tokens.last() == Some(&Token::CloseParentheses)
    {
        return parse_expression_tokens(&tokens[1..tokens.len() - 1]);
    }

    // codeblock
    if tokens.first() == Some(&Token::OpenCurlyBrackets)
        && tokens.last() == Some(&Token::CloseCurlyBrackets)
    {
        let expressions = identify_expressions(&tokens[1..tokens.len() - 1])?;
        let mut statements = vec![];
        for expression in expressions {
            statements.push(parse_expression_tokens(expression)?)
        }
        return Ok(KodyNode::CodeBlock { statements });
    }

    // return
    if tokens.first() == Some(&Token::Return) {
        let return_value = if tokens.len() == 1 {
            Box::new(KodyNode::GetConstant { id: 0 })
        } else {
            Box::new(parse_expression_tokens(&tokens[1..tokens.len()])?)
        };
        return Ok(KodyNode::ReturnFromFunction { return_value });
    }

    // if expression
    if tokens.first() == Some(&Token::If) {
        let (condition_tokens, other_tokens) = get_next_expression(&tokens[1..tokens.len()])?;
        let (action_tokens, other_tokens) = get_next_expression(&other_tokens)?;
        let else_action = if other_tokens.first() == Some(&Token::Else) {
            Some(Box::new(parse_expression_tokens(
                &other_tokens[1..other_tokens.len()],
            )?))
        } else {
            None
        };
        let condition = parse_expression_tokens(condition_tokens)?;
        let action = parse_expression_tokens(action_tokens)?;
        return Ok(KodyNode::IfStatement {
            condition: Box::new(condition),
            action: Box::new(action),
            else_action,
        });
    }

    // while expression
    if tokens.first() == Some(&Token::While) {
        let (condition_tokens, other_tokens) = get_next_expression(&tokens[1..tokens.len()])?;
        let (action_tokens, _) = get_next_expression(&other_tokens)?;
        let condition = parse_expression_tokens(condition_tokens)?;
        let action = parse_expression_tokens(action_tokens)?;
        return Ok(KodyNode::WhileStatement {
            condition: Box::new(condition),
            action: Box::new(action),
        });
    }

    if tokens.first() == Some(&Token::Subtract) {
        return Ok(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetConstant { id: 0 }),
            arguments: vec![parse_expression_tokens(&tokens[1..])?],
        });
    }

    if tokens.len() == 1 {
        match tokens.first().unwrap() {
            Token::Identifier(_name) => return Ok(KodyNode::GetVariable { name_id: 0 }),
            Token::StringLiteral(_value) => return Ok(KodyNode::GetConstant { id: 0 }),
            Token::Number(_value) => return Ok(KodyNode::GetConstant { id: 0 }),
            Token::True => return Ok(KodyNode::GetConstant { id: 0 }),
            Token::False => return Ok(KodyNode::GetConstant { id: 0 }),
            _ => unreachable!(),
        }
    }

    // assignment operators
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Assign
            | Token::AddAssign
            | Token::SubtractAssign
            | Token::MultiplyAssign
            | Token::DivideAssign => {
                let (variable_tokens, mut value_tokens) = tokens.split_at(i);
                value_tokens = match value_tokens.split_first() {
                    Some((_first, rest)) => rest,
                    None => return Err(String::from("No value after assign operator!")),
                };
                return match token {
                    Token::Assign => Ok(KodyNode::SetVariable {
                        variable: Box::new(parse_expression_tokens(variable_tokens)?),
                        value: Box::new(parse_expression_tokens(value_tokens)?),
                    }),
                    Token::AddAssign => Ok(KodyNode::SetVariable {
                        variable: Box::new(parse_expression_tokens(variable_tokens)?),
                        value: Box::new(KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![
                                parse_expression_tokens(variable_tokens)?,
                                parse_expression_tokens(value_tokens)?,
                            ],
                        }),
                    }),
                    Token::SubtractAssign => Ok(KodyNode::SetVariable {
                        variable: Box::new(parse_expression_tokens(variable_tokens)?),
                        value: Box::new(KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![
                                parse_expression_tokens(variable_tokens)?,
                                parse_expression_tokens(value_tokens)?,
                            ],
                        }),
                    }),
                    Token::MultiplyAssign => Ok(KodyNode::SetVariable {
                        variable: Box::new(parse_expression_tokens(variable_tokens)?),
                        value: Box::new(KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![
                                parse_expression_tokens(variable_tokens)?,
                                parse_expression_tokens(value_tokens)?,
                            ],
                        }),
                    }),
                    Token::DivideAssign => Ok(KodyNode::SetVariable {
                        variable: Box::new(parse_expression_tokens(variable_tokens)?),
                        value: Box::new(KodyNode::CallFunction {
                            function: Box::new(KodyNode::GetConstant { id: 0 }),
                            arguments: vec![
                                parse_expression_tokens(variable_tokens)?,
                                parse_expression_tokens(value_tokens)?,
                            ],
                        }),
                    }),
                    _ => unreachable!(),
                };
            }
            _ => (),
        }
    }

    // comparison operators
    for (i, token) in tokens.iter().enumerate() {
        if tokens
            .split_at(i)
            .0
            .iter()
            .filter(|t| *t == &Token::OpenParentheses || *t == &Token::OpenCurlyBrackets)
            .count()
            != tokens
                .split_at(i)
                .0
                .iter()
                .filter(|t| *t == &Token::CloseParentheses || *t == &Token::CloseCurlyBrackets)
                .count()
        {
            continue;
        }
        match token {
            Token::Equals => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::NotEqual => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::GreaterThan => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::GreaterThanOrEqual => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::LessThan => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::LessThanOrEqual => {
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            _ => (),
        }
    }

    // add and subtract operators
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Add => {
                if tokens.get(i + 1) == Some(&Token::Add)
                    || tokens.get(i + 1) == Some(&Token::Subtract)
                {
                    return Err(String::from(
                        "Two consecutive addition or subtraction symbols",
                    ));
                }
                if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                    && tokens.split_at(i).1.contains(&Token::CloseParentheses)
                {
                    continue;
                }
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::Subtract => {
                if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                    && tokens.split_at(i).1.contains(&Token::CloseParentheses)
                {
                    continue;
                }
                if tokens.get(i + 1) == Some(&Token::Add)
                    || tokens.get(i + 1) == Some(&Token::Subtract)
                {
                    return Err(String::from(
                        "Two consecutive addition or subtraction symbols",
                    ));
                }
                if match tokens.get(i - 1).unwrap_or(&Token::OpenCurlyBrackets) {
                    Token::Multiply
                    | Token::Divide
                    | Token::AddAssign
                    | Token::SubtractAssign
                    | Token::MultiplyAssign
                    | Token::DivideAssign
                    | Token::OpenParentheses
                    | Token::OpenCurlyBrackets
                    | Token::If
                    | Token::Else
                    | Token::While
                    | Token::And
                    | Token::Or
                    | Token::Not
                    | Token::True
                    | Token::False
                    | Token::Return
                    | Token::Assign
                    | Token::Equals
                    | Token::NotEqual
                    | Token::GreaterThan
                    | Token::LessThan
                    | Token::GreaterThanOrEqual
                    | Token::LessThanOrEqual
                    | Token::Separator => false,
                    _ => true,
                } {
                    return Ok(KodyNode::CallFunction {
                        function: Box::new(KodyNode::GetConstant { id: 0 }),
                        arguments: vec![
                            parse_expression_tokens(tokens.split_at(i).0)?,
                            parse_expression_tokens(tokens.split_at(i + 1).1)?,
                        ],
                    });
                }
            }
            _ => (),
        }
    }

    // multiply and divide operators
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Multiply => {
                if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                    && tokens.split_at(i).1.contains(&Token::CloseParentheses)
                {
                    continue;
                }
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            Token::Divide => {
                if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                    && tokens.split_at(i).1.contains(&Token::CloseParentheses)
                {
                    continue;
                }
                return Ok(KodyNode::CallFunction {
                    function: Box::new(KodyNode::GetConstant { id: 0 }),
                    arguments: vec![
                        parse_expression_tokens(tokens.split_at(i).0)?,
                        parse_expression_tokens(tokens.split_at(i + 1).1)?,
                    ],
                });
            }
            _ => (),
        }
    }

    for (i, token) in tokens.iter().enumerate() {
        if *token == Token::Not {
            if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                && tokens.split_at(i).1.contains(&Token::CloseParentheses)
            {
                continue;
            }
            return Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![parse_expression_tokens(&tokens[1..tokens.len()])?],
            });
        }
    }

    for (i, token) in tokens.iter().enumerate() {
        if *token == Token::Not {
            if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                && tokens.split_at(i).1.contains(&Token::CloseParentheses)
            {
                continue;
            }
            return Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![parse_expression_tokens(&tokens[1..tokens.len()])?],
            });
        }
    }

    for (i, token) in tokens.iter().enumerate() {
        if *token == Token::Not {
            if tokens.split_at(i).0.contains(&Token::OpenParentheses)
                && tokens.split_at(i).1.contains(&Token::CloseParentheses)
            {
                continue;
            }
            return Ok(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetConstant { id: 0 }),
                arguments: vec![parse_expression_tokens(&tokens[1..tokens.len()])?],
            });
        }
    }

    for (i, token) in tokens.iter().rev().enumerate() {
        match token {
            Token::CloseParentheses => {
                let mut indent_level = 0;
                let mut open_parentheses_index = 0;
                for (i, token) in tokens.iter().enumerate() {
                    match token {
                        Token::OpenParentheses => {
                            if indent_level > 0 {
                                indent_level += 1
                            } else {
                                open_parentheses_index = i;
                                break;
                            }
                        }
                        Token::CloseParentheses => indent_level -= 1,
                        _ => (),
                    }
                }

                if open_parentheses_index == 0 {
                    return Err(String::from("Can't find pair for closing parentheses!"));
                }

                let (function_tokens, mut argument_tokens) =
                    tokens.split_at(open_parentheses_index);

                argument_tokens = &argument_tokens[1..argument_tokens.len() - 1];

                let mut separator_indices = vec![];

                indent_level = 0;

                for (i, token) in argument_tokens.iter().enumerate() {
                    match token {
                        Token::OpenParentheses => indent_level += 1,
                        Token::CloseParentheses => indent_level -= 1,
                        Token::Separator => {
                            if indent_level == 0 {
                                separator_indices.push(i)
                            }
                        }
                        _ => (),
                    }
                }

                let mut arguments = vec![];
                if !argument_tokens.is_empty() {
                    let mut last_index = 0;
                    for index in separator_indices {
                        arguments.push(parse_expression_tokens(
                            &argument_tokens[last_index..index],
                        )?);

                        last_index = index + 1;
                    }
                    arguments.push(parse_expression_tokens(
                        &argument_tokens[last_index..argument_tokens.len()],
                    )?);
                }

                return Ok(KodyNode::CallFunction {
                    function: Box::new(parse_expression_tokens(function_tokens)?),
                    arguments,
                });
            }

            Token::MemberAcces => {
                if let Some(&Token::Identifier(_)) = tokens.get(tokens.len() - i) {
                    return Ok(KodyNode::GetMember {
                        base_object: Box::new(parse_expression_tokens(
                            tokens.split_at(tokens.len() - i - 1).0,
                        )?),
                        member_name_id: 0,
                    });
                }
            }
            _ => (),
        }
    }

    unreachable!()
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
}
