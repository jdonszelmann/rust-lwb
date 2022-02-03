use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::typechecker::state::State;
use std::marker::PhantomData;

pub mod constraints;
pub mod state;

pub trait Type {}

pub trait TypeCheckable<M: AstInfo, CTX, TYPE: Type>: AstNode<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, CTX, TYPE>, ctx: &CTX);
}

impl<M: AstInfo, CTX, TYPE: Type, T> TypeCheckable<M, CTX, TYPE> for Box<T>
where
    T: TypeCheckable<M, CTX, TYPE>,
{
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, CTX, TYPE>, ctx: &CTX) {
        T::create_constraints(self, s, ctx)
    }
}

pub struct TypeChecker<M, CTX, TYPE> {
    phantom: PhantomData<(M, CTX, TYPE)>,
}

impl<M, CTX, TYPE> Default for TypeChecker<M, CTX, TYPE> {
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}


impl<M: AstInfo, CTX, TYPE: Type> TypeChecker<M, CTX, TYPE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn check_types<Ast>(self, ast: Ast, ctx: &CTX)
    where
        Ast: TypeCheckable<M, CTX, TYPE>,
    {
        let mut state = State::new();
        state.type_ok(&ast);

        while let Some(i) = state.next_node_to_typecheck() {
            i.create_constraints(&mut state, ctx);
        }
    }
}
