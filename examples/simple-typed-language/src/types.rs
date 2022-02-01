use rust_lwb::codegen_prelude::AstInfo;
use rust_lwb::typechecker::constraints::{Constraint, ConstraintBuilder};
use rust_lwb::typechecker::{Type, TypeCheckable};
use crate::stl::Program;

#[derive(Eq, PartialEq, Debug)]
pub enum StlType {
    Int
}

impl Type for StlType {}


impl<M: AstInfo> TypeCheckable<M, ()> for Program<M> {
    type Type = StlType;

    fn generate_constraints(&self, cb: ConstraintBuilder, ctx: ()) -> Constraint<Self::Type> {
        todo!()
    }
}