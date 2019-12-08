pub mod objects;

use std::collections::HashMap;

use crate::syntax_tree::{KodyNode, KodySyntaxTree};
use objects::{KodyObject, KodyValue};

pub fn execute(syntax_tree: &KodySyntaxTree) -> Result<(), String> {
    let global_variables = syntax_tree
        .functions
        .iter()
        .map(|func_data| {
            (
                func_data.name.clone(),
                KodyObject::from(KodyValue::Function(func_data.clone())),
            )
        })
        .collect(); // make a hashmap out of provided functions

    let mut variable_stack = VariableStack::new(global_variables); // start with global variables

    execute_node(&syntax_tree.main, &mut variable_stack)?;
    Ok(())
}

struct VariableStack {
    closures: Vec<HashMap<String, KodyObject>>,
}

impl VariableStack {
    pub fn new(global_variables: HashMap<String, KodyObject>) -> VariableStack {
        VariableStack {
            closures: vec![global_variables],
        }
    }
    // TODO is there a better way to do this?
    pub fn open_closure(&mut self) {
        self.closures.push(HashMap::new());
    }

    pub fn close_closure(&mut self) {
        self.closures.pop();
    }
    pub fn set(&mut self, name: &str, value: KodyObject) {
        println!("Setting {} to {:?}", name, value);
    }

    pub fn get(&self, name: &str) -> Result<KodyObject, String> {
        for closure in &self.closures {
            if let Some(value) = closure.get(name) {
                return Ok(value.clone());
            }
        }
        Err(String::from(
            "Variable name doesn't match any known variable!",
        ))
    }
}

fn execute_node(node: &KodyNode, variable_stack: &mut VariableStack) -> Result<KodyObject, String> {
    match node {
        KodyNode::CodeBlock { statements } => return execute_codeblock(statements, variable_stack),
        KodyNode::GetConstant { value } => return Ok(value.clone()),
        KodyNode::GetVariable { name } => return variable_stack.get(name),
        KodyNode::SetVariable { name, value } => {
            let value_object = execute_node(value, variable_stack)?;
            variable_stack.set(name, value_object);
        }
        _ => println!("{:?}", node),
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
