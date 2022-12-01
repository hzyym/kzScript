use crate::ast::node::{Expression, Statement};

#[derive(Debug,PartialEq)]
pub struct IfStatement {
    pub condition:Expression,
    pub consequence:Vec<Statement>,
    pub alternative:Vec<Statement>,
}

impl IfStatement {
    pub fn new(condition:Expression) -> IfStatement {
        Self {
            condition,
            consequence:Vec::new(),
            alternative:Vec::new(),
        }
    }
}