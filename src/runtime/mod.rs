
pub mod objects;

use crate::syntax_tree::{KodySyntaxTree, KodyNode, KodyFunctionData};

pub fn execute(syntax_tree: &KodySyntaxTree) -> Result<(), String> {
    execute_node(&syntax_tree.main, &syntax_tree.functions)?;
    Ok(())
}

fn execute_node(node: &KodyNode, functions: &[KodyFunctionData]) -> Result<(), String> {
    match node {
        KodyNode::CodeBlock{ statements } => {
            for statement in statements {
                execute_node(statement, functions)?;
            }
        }
        _ => println!("{:?}", node),
    }
    Ok(())
}