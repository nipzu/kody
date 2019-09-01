use crate::syntax_tree::KodyNode;

pub fn add_function(name: &str, arguments: Vec<&str>, body: Box<KodyNode>) {
    println!(
        "added function: {:?} with arguments: {:?} and body: {:?}",
        name, arguments, body
    );
}