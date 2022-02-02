use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use crate::codegen_prelude::AstInfo;
use crate::parser::ast::{AstNode, NodeId};
use crate::typechecker::constraints::{Constraint, FreeVariable, KnownVariable, Variable, VariableId};
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::state::State;

pub mod constraints;
pub mod state;

pub trait Type {}

pub trait TypeCheckable<M: AstInfo, CTX, TYPE: Type>: AstNode<M> {
    fn create_constraints(&self, s: &mut State<M, CTX, TYPE>, ctx: &CTX);
}

impl<M: AstInfo, CTX, TYPE: Type, T> TypeCheckable<M, CTX, TYPE> for Box<T> where T: TypeCheckable<M, CTX, TYPE> {
    fn create_constraints(&self, s: &mut State<M, CTX, TYPE>, ctx: &CTX) {
        TypeCheckable::create_constraints(self, s, ctx)
    }
}

pub struct TypeChecker<M, CTX, TYPE> {
    phantom: PhantomData<(M, CTX, TYPE)>,
}

impl<M: AstInfo, CTX, TYPE: Type> TypeChecker<M, CTX, TYPE> {
    pub fn new() -> Self {
        Self {
            phantom: Default::default()
        }
    }

    pub fn check_types<Ast>(self, ast: Ast, ctx: &CTX)
        where Ast: TypeCheckable<M, CTX, TYPE> {

        let (constraints, variables) = self.collect_constraints();
        // let mut state = State::new();
        //
        // ast.create_constraints(&mut state, ctx)
    }

    fn collect_constraints(&self) -> (Vec<Constraint<TYPE>>, Vec<Variable<TYPE>>) {
        todo!()
    }
}

