use crate::stl::{Program, Statement};
use crate::AST::Expression;
use rust_lwb::codegen_prelude::AstInfo;
use rust_lwb::typechecker::state::State;
use rust_lwb::typechecker::{Type, TypeCheckable};

#[derive(Eq, PartialEq, Debug)]
pub enum StlType {
    Int,
    Bool,
}

impl Type for StlType {}

impl<M: AstInfo> TypeCheckable<M, (), StlType> for Program<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        match self {
            Program::Program(_, statements) => {
                for i in statements {
                    s.type_ok(i);
                }
            }
        }
    }
}

impl<M: AstInfo> TypeCheckable<M, (), StlType> for Statement<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        match self {
            Statement::If(_, e, block) => {
                let te = s.get_type(e);
                s.add_constraint(te.equiv(StlType::Bool));

                for i in block {
                    s.type_ok(i);
                }
            }
            Statement::Expression(_, e) => s.type_ok(e),
            Statement::Assignment(_, _, e) => {
                // something with scopes: TODO
                s.type_ok(e)
            }
        }
    }
}

impl<M: AstInfo> TypeCheckable<M, (), StlType> for Expression<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        match self {
            Expression::Add(_, a, b) => {
                let ta = s.get_type(a);
                let tb = s.get_type(b);

                s.add_constraint(ta.equiv(tb));
                s.type_of_self(self).equiv(ta);
            }
            Expression::Sub(_, a, b) => {
                let ta = s.get_type(a);
                let tb = s.get_type(b);

                s.add_constraint(ta.equiv(tb));
                s.type_of_self(self).equiv(ta);
            }
            Expression::Int(_, _a) => {
                s.type_of_self(self).equiv(StlType::Int);
            }
            Expression::Identifier(_, _s) => {
                // something with scopes: TODO
            }
        }
    }
}
