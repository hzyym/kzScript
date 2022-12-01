use crate::ast::node::{Expression, Statement};

#[derive(Debug,PartialEq)]
pub struct ForStatement {
    pub start_condition:Box<Statement>,
    pub condition:Expression,
    pub self_operation:Box<Statement>,
    pub consequence:Vec<Statement>,
}
impl ForStatement {
    pub fn new(start_condition:Statement,exp:Expression,self_op:Statement) -> ForStatement {
        Self {
            start_condition:Box::new(start_condition),
            condition:exp,
            self_operation:Box::new(self_op),
            consequence:vec![]
        }
    }
}