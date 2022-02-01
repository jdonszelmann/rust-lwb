use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::typechecker::constraints::{Constraint, ConstraintBuilder};

mod constraints;

pub trait Type {}

pub trait TypeCheckable<M: AstInfo, CTX>: AstNode<M> {
    type Type: Type;

    fn generate_constraints(&self, cb: ConstraintBuilder, ctx: CTX) -> Constraint<Self::Type>;
}

pub struct Solver<TYPE: Type> {
    constraints: Constraint<TYPE>,
}

impl<TYPE: Type> Solver<TYPE> {
    pub fn solve<CTX, M: AstInfo>(&mut self, root: impl TypeCheckable<M, CTX>, ctx: CTX) {
        self.collect_constraints(root, ctx);
        self.solve_constraints();
    }

    fn collect_constraints<CTX, M: AstInfo>(&mut self, root: impl TypeCheckable<M, CTX>, ctx: CTX) {
        todo!()
    }

    fn solve_constraints(&mut self) {
        todo!()
    }
}
