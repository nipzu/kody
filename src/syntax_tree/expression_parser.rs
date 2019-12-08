use super::{get_next_expression, identify_expressions, KodyNode};
use crate::runtime::objects::{KodyObject, KodyValue};
use crate::tokenizer::Token;

fn is_in_codeblock(tokens: &[Token], index: usize) -> bool {
    tokens
        .split_at(index)
        .0
        .iter()
        .filter(|t| *t == &Token::OpenCurlyBrackets)
        .count()
        > tokens
            .split_at(index)
            .0
            .iter()
            .filter(|t| *t == &Token::CloseCurlyBrackets)
            .count()
}

fn is_in_parentheses(tokens: &[Token], index: usize) -> bool {
    tokens
        .split_at(index)
        .0
        .iter()
        .filter(|t| *t == &Token::OpenParentheses)
        .count()
        > tokens
            .split_at(index)
            .0
            .iter()
            .filter(|t| *t == &Token::CloseParentheses)
            .count()
}

fn check_parentheses(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.first() == Some(&Token::OpenParentheses)
        && tokens.last() == Some(&Token::CloseParentheses)
    {
        return parse_expression_tokens(&tokens[1..tokens.len() - 1]).map(Some);
    }
    Ok(None)
}

fn check_codeblock(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.first() == Some(&Token::OpenCurlyBrackets)
        && tokens.last() == Some(&Token::CloseCurlyBrackets)
    {
        let expressions = identify_expressions(&tokens[1..tokens.len() - 1])?;
        let mut statements = vec![];
        for expression in expressions {
            statements.push(parse_expression_tokens(expression)?);
        }
        return Ok(Some(KodyNode::CodeBlock { statements }));
    }
    Ok(None)
}

fn check_return(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.first() == Some(&Token::Return) {
        let return_value = if tokens.len() == 1 {
            Box::new(KodyNode::GetConstant {
                value: KodyObject::from(KodyValue::Empty),
            })
        } else {
            Box::new(parse_expression_tokens(&tokens[1..tokens.len()])?)
        };
        return Ok(Some(KodyNode::ReturnFromFunction { return_value }));
    }
    Ok(None)
}

fn check_if_expression(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
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
        return Ok(Some(KodyNode::IfStatement {
            condition: Box::new(condition),
            action: Box::new(action),
            else_action,
        }));
    }
    Ok(None)
}

fn check_while_expression(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.first() == Some(&Token::While) {
        let (condition_tokens, other_tokens) = get_next_expression(&tokens[1..tokens.len()])?;
        let (action_tokens, _) = get_next_expression(&other_tokens)?;
        let condition = parse_expression_tokens(condition_tokens)?;
        let action = parse_expression_tokens(action_tokens)?;
        return Ok(Some(KodyNode::WhileStatement {
            condition: Box::new(condition),
            action: Box::new(action),
        }));
    }
    Ok(None)
}

fn check_negation(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.first() == Some(&Token::Subtract) {
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: String::from("__negate"),
            }),
            arguments: vec![parse_expression_tokens(&tokens[1..])?],
        }));
    }
    Ok(None)
}

fn check_value(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if tokens.len() == 1 {
        return Ok(Some(match tokens.first().unwrap() {
            Token::Identifier(name) => KodyNode::GetVariable { name: name.clone() },
            Token::StringLiteral(value) => KodyNode::GetConstant {
                value: KodyObject::from(KodyValue::StringLiteral(value.clone())),
            },
            Token::Number(value) => KodyNode::GetConstant {
                value: KodyObject::from(KodyValue::Number(value.clone())),
            },
            Token::True => KodyNode::GetConstant {
                value: KodyObject::from(KodyValue::Bool(true)),
            },
            Token::False => KodyNode::GetConstant {
                value: KodyObject::from(KodyValue::Bool(false)),
            },
            _ => unreachable!(),
        }));
    }
    Ok(None)
}

fn check_assignment(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Assign
        | Token::AddAssign
        | Token::SubtractAssign
        | Token::MultiplyAssign
        | Token::DivideAssign => !is_in_codeblock(tokens, index),
        _ => false,
    }) {
        let (variable_tokens, mut value_tokens) = tokens.split_at(i);
        // remove the assignment operator
        value_tokens = match value_tokens.split_first() {
            Some((_first, rest)) => rest,
            None => return Err(String::from("No value after assign operator!")),
        };

        let variable_name = if let [Token::Identifier(identifier)] = variable_tokens {
            identifier.clone()
        } else {
            return Err(String::from("Cannot assign to a non-identifier variable!"));
        };

        if tokens[i] == Token::Assign {
            return Ok(Some(KodyNode::SetVariable {
                name: variable_name,
                value: Box::new(parse_expression_tokens(value_tokens)?),
            }));
        }

        let function_name = match tokens[i] {
            Token::AddAssign => "__add",
            Token::SubtractAssign => "__subtract",
            Token::MultiplyAssign => "__multiply",
            Token::DivideAssign => "__divide",
            _ => unreachable!(),
        };

        return Ok(Some(KodyNode::SetVariable {
            name: variable_name,
            value: Box::new(KodyNode::CallFunction {
                function: Box::new(KodyNode::GetVariable {
                    name: function_name.to_string(),
                }),
                arguments: vec![
                    parse_expression_tokens(variable_tokens)?,
                    parse_expression_tokens(value_tokens)?,
                ],
            }),
        }));
    }
    Ok(None)
}

fn check_comparison(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Equals
        | Token::NotEqual
        | Token::GreaterThanOrEqual
        | Token::GreaterThan
        | Token::LessThanOrEqual
        | Token::LessThan => !is_in_codeblock(tokens, index) && !is_in_parentheses(tokens, index),
        _ => false,
    }) {
        let function_name = match tokens[i] {
            Token::Equals => "__equals",
            Token::NotEqual => "__not_equals",
            Token::GreaterThan => "__greater_than",
            Token::GreaterThanOrEqual => "__greater_than_or_equal",
            Token::LessThan => "__less_than",
            Token::LessThanOrEqual => "__less_than_or_equal",
            _ => unreachable!(),
        };
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: function_name.to_string(),
            }),
            arguments: vec![
                parse_expression_tokens(tokens.split_at(i).0)?,
                parse_expression_tokens(tokens.split_at(i + 1).1)?,
            ],
        }));
    }
    Ok(None)
}

fn check_addition_and_subtraction(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Add | Token::Subtract => {
            !is_in_codeblock(tokens, index)
                && !is_in_parentheses(tokens, index)
                && index != 0
                && match tokens[index - 1] {
                    Token::Number(_)
                    | Token::Identifier(_)
                    | Token::StringLiteral(_)
                    | Token::CloseParentheses
                    | Token::CloseCurlyBrackets => true,
                    _ => false,
                }
        }
        _ => false,
    }) {
        if tokens.get(i + 1) == Some(&Token::Add) || tokens.get(i + 1) == Some(&Token::Subtract) {
            return Err(String::from(
                "Two consecutive addition or subtraction symbols",
            ));
        }
        let function_name = match tokens[i] {
            Token::Add => "__add",
            Token::Subtract => "__subtract",
            _ => unreachable!(),
        };
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: function_name.to_string(),
            }),
            arguments: vec![
                parse_expression_tokens(tokens.split_at(i).0)?,
                parse_expression_tokens(tokens.split_at(i + 1).1)?,
            ],
        }));
    }
    Ok(None)
}

fn check_multiplication_and_division(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Multiply | Token::Divide => {
            !is_in_codeblock(tokens, index) && !is_in_parentheses(tokens, index)
        }
        _ => false,
    }) {
        let function_name = match tokens[i] {
            Token::Multiply => "__multiply",
            Token::Divide => "__divide",
            _ => unreachable!(),
        };
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: function_name.to_string(),
            }),
            arguments: vec![
                parse_expression_tokens(tokens.split_at(i).0)?,
                parse_expression_tokens(tokens.split_at(i + 1).1)?,
            ],
        }));
    }
    Ok(None)
}

fn check_or_operator(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Or => !is_in_codeblock(tokens, index) && !is_in_parentheses(tokens, index),
        _ => false,
    }) {
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: String::from("__or"),
            }),
            arguments: vec![
                parse_expression_tokens(tokens.split_at(i).0)?,
                parse_expression_tokens(tokens.split_at(i + 1).1)?,
            ],
        }));
    }
    Ok(None)
}

fn check_and_operator(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::And => !is_in_codeblock(tokens, index) && !is_in_parentheses(tokens, index),
        _ => false,
    }) {
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: String::from("__and"),
            }),
            arguments: vec![
                parse_expression_tokens(tokens.split_at(i).0)?,
                parse_expression_tokens(tokens.split_at(i + 1).1)?,
            ],
        }));
    }
    Ok(None)
}

fn check_not_operator(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    if let Some(i) = tokens.iter().enumerate().position(|(index, t)| match t {
        Token::Not => !is_in_codeblock(tokens, index) && !is_in_parentheses(tokens, index),
        _ => false,
    }) {
        return Ok(Some(KodyNode::CallFunction {
            function: Box::new(KodyNode::GetVariable {
                name: String::from("__not"),
            }),
            arguments: vec![parse_expression_tokens(tokens.split_at(i + 1).1)?],
        }));
    }
    Ok(None)
}

// TODO idk if you can improve this
fn check_function_call_and_member_access(tokens: &[Token]) -> Result<Option<KodyNode>, String> {
    // _i for memberaccess
    for (_i, token) in tokens.iter().rev().enumerate() {
        match token {
            Token::CloseParentheses => {
                let mut indent_level = 1;
                let mut open_parentheses_index = 0;
                for (i, token) in tokens.iter().rev().enumerate().skip(1) {
                    match token {
                        Token::OpenParentheses => {
                            indent_level -= 1;
                            if indent_level == 0 {
                                open_parentheses_index = tokens.len() - 1 - i;
                                break;
                            }
                        }
                        Token::CloseParentheses => indent_level += 1,
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

                return Ok(Some(KodyNode::CallFunction {
                    function: Box::new(parse_expression_tokens(function_tokens)?),
                    arguments,
                }));
            }

            Token::MemberAccess => {
                unimplemented!();
                // see above row
                /*if let Some(Token::Identifier(member_name)) = tokens.get(tokens.len() - i) {
                    return Ok(Some(KodyNode::GetMember {
                        base_object: Box::new(parse_expression_tokens(
                            tokens.split_at(tokens.len() - i - 1).0,
                        )?),
                        member_name: member_name.clone(),
                    }));
                }*/
            }
            _ => (),
        }
    }
    Ok(None)
}

pub fn parse_expression_tokens(tokens: &[Token]) -> Result<KodyNode, String> {
    assert!(!tokens.is_empty());

    for check in &[
        check_parentheses,
        check_codeblock,
        check_return,
        check_if_expression,
        check_while_expression,
        check_negation,
        check_value,
        check_assignment,
        check_comparison,
        check_addition_and_subtraction,
        check_multiplication_and_division,
        check_or_operator,
        check_and_operator,
        check_not_operator,
        check_function_call_and_member_access,
    ] {
        match check(tokens) {
            Ok(Some(node)) => return Ok(node),
            Err(e) => return Err(e),
            Ok(None) => (),
        }
    }

    unreachable!()
}
