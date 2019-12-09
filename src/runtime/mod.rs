pub mod objects;

use std::collections::HashMap;

use crate::libkody::GLOBALS;
use crate::syntax_tree::{KodyNode, KodySyntaxTree};
use objects::{KodyObject, KodyValue};

pub fn execute(syntax_tree: &KodySyntaxTree) -> Result<KodyObject, String> {
    let mut variable_stack = VariableStack::new(syntax_tree.global_variables.clone()); // start with global variables

    execute_node(&syntax_tree.main, &mut variable_stack)?;

    Ok(variable_stack.return_value.unwrap_or_else(KodyObject::new))
}

struct VariableStack {
    closures: Vec<HashMap<String, KodyObject>>,
    return_value: Option<KodyObject>,
    // this will be set to Some(value) when a function returns
    // and then passed through
}

impl VariableStack {
    pub fn new(global_variables: HashMap<String, KodyObject>) -> VariableStack {
        VariableStack {
            closures: vec![global_variables],
            return_value: None,
        }
    }
    // TODO is there a better way to do this?
    pub fn open_closure(&mut self) {
        self.closures.push(HashMap::new());
    }

    pub fn close_closure(&mut self) {
        self.closures.pop();
    }
    pub fn set(&mut self, name: &str, new_value: KodyObject) {
        for closure in self.closures.iter_mut().rev() {
            if let Some(value) = closure.get_mut(name) {
                *value = new_value;
                return;
            }
        }

        // If no variable was found, create a new one
        // TODO is it safe to unwrap
        self.closures
            .last_mut()
            .unwrap()
            .insert(name.to_string(), new_value);
    }

    pub fn get(&self, name: &str) -> Result<KodyObject, String> {
        for closure in self.closures.iter().rev() {
            if let Some(value) = closure.get(name) {
                return Ok(value.clone());
            }
        }

        if let Some(value) = GLOBALS.get(name) {
            return Ok(value.clone());
        }

        Err(format!(
            "Variable name {} doesn't match any known variable!",
            name
        ))
    }
}

fn execute_node(node: &KodyNode, variable_stack: &mut VariableStack) -> Result<KodyObject, String> {
    // propagate return value
    if variable_stack.return_value.is_some() {
        return Ok(KodyObject::new());
    }

    match node {
        KodyNode::CodeBlock { statements } => return execute_codeblock(statements, variable_stack),
        KodyNode::GetConstant { value } => return Ok(value.clone()),
        KodyNode::GetVariable { name } => return variable_stack.get(name),
        KodyNode::SetVariable { name, value } => {
            let value_object = execute_node(value, variable_stack)?;
            variable_stack.set(name, value_object);
        }
        KodyNode::IfStatement {
            condition,
            action,
            else_action,
        } => return execute_if_statement(condition, action, else_action, variable_stack),
        KodyNode::WhileStatement { condition, action } => {
            return execute_while_statement(condition, action, variable_stack)
        }
        KodyNode::ReturnFromFunction { return_value } => {
            variable_stack.return_value = Some(execute_node(return_value, variable_stack)?)
        }
        KodyNode::CallFunction {
            function,
            arguments,
        } => return execute_function_call(function, arguments, variable_stack),
    }
    Ok(KodyObject::new())
}

fn execute_codeblock(
    statements: &[KodyNode],
    variable_stack: &mut VariableStack,
) -> Result<KodyObject, String> {
    variable_stack.open_closure(); // add a new closure

    for statement in statements {
        execute_node(statement, variable_stack)?; // execute every statement
    }

    variable_stack.close_closure(); // delete variables from closure
    Ok(KodyObject::new())
}

fn execute_if_statement(
    condition: &KodyNode,
    action: &KodyNode,
    else_action: &Option<Box<KodyNode>>,
    variable_stack: &mut VariableStack,
) -> Result<KodyObject, String> {
    if match *execute_node(condition, variable_stack)?.value {
        KodyValue::Bool(true) => true,
        KodyValue::Bool(false) => false,
        _ => return Err(String::from("Object in if condition was not a bool!")),
    } {
        execute_node(action, variable_stack)?;
    } else if let Some(node) = else_action {
        execute_node(node, variable_stack)?;
    }

    Ok(KodyObject::new())
}

fn execute_while_statement(
    condition: &KodyNode,
    action: &KodyNode,
    variable_stack: &mut VariableStack,
) -> Result<KodyObject, String> {
    while match *execute_node(condition, variable_stack)?.value {
        KodyValue::Bool(true) => true,
        KodyValue::Bool(false) => false,
        _ => return Err(String::from("Object in while condition was not a bool!")),
    } {
        execute_node(action, variable_stack)?;
    }

    Ok(KodyObject::new())
}

fn execute_function_call(
    function: &KodyNode,
    arguments: &[KodyNode],
    variable_stack: &mut VariableStack,
) -> Result<KodyObject, String> {
    if let KodyValue::Function(func_data) = *execute_node(function, variable_stack)?.value {
        if func_data.arguments.len() != arguments.len() {
            return Err(String::from(
                "Different number of arguments in function definition and function call!",
            ));
        }

        let mut argument_objects = Vec::new();

        for arg in arguments {
            argument_objects.push(execute_node(arg, variable_stack)?);
        }

        let mut new_globals = variable_stack.closures[0].clone();
        new_globals.extend(
            func_data
                .arguments
                .iter()
                .zip(argument_objects.iter())
                .map(|(name, object)| (name.clone(), object.clone())),
        );

        let tree = KodySyntaxTree {
            global_variables: new_globals,
            main: func_data.body,
        };

        execute(&tree)
    } else if let KodyValue::NativeFunction(function) =
        *execute_node(function, variable_stack)?.value
    {
        let mut argument_objects = Vec::new();
        for arg in arguments {
            argument_objects.push(execute_node(arg, variable_stack)?);
        }

        function(argument_objects)
    } else {
        Err(String::from(
            "Cannot make function call with value other than a function",
        ))
    }
}
