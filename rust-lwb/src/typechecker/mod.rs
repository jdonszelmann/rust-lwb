use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::error::TypeError;
use crate::typechecker::solver::Solver;
use crate::typechecker::state::State;
use itertools::Itertools;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod constraints;
pub mod error;
mod solver;
pub mod state;
mod union_find;

pub trait Type: Debug + PartialEq {}

pub trait TypeCheckable<M: AstInfo, CTX, TYPE: Type, ERR>: AstNode<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, CTX, TYPE, ERR>, ctx: &CTX);
}

impl<M: AstInfo, CTX, TYPE: Type, ERR, T> TypeCheckable<M, CTX, TYPE, ERR> for Box<T>
where
    T: TypeCheckable<M, CTX, TYPE, ERR>,
{
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, CTX, TYPE, ERR>, ctx: &CTX) {
        T::create_constraints(self, s, ctx)
    }
}

pub struct TypeChecker<M, CTX, TYPE, ERR> {
    phantom: PhantomData<(M, CTX, TYPE, ERR)>,
}

impl<M, CTX, TYPE, ERR> Default for TypeChecker<M, CTX, TYPE, ERR> {
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<M: AstInfo, CTX, TYPE: Type, ERR> TypeChecker<M, CTX, TYPE, ERR> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn check_types<Ast>(self, ast: Ast, ctx: &CTX) -> Result<(), TypeError<TYPE, ERR>>
    where
        Ast: TypeCheckable<M, CTX, TYPE, ERR>,
    {
        let mut state = State::new();
        state.type_ok(&ast);

        let mut had = HashSet::new();

        while let Some(i) = state.todo.pop_front() {
            let nodeid = i.ast_info().node_id();
            if had.contains(&nodeid) {
                continue;
            }

            had.insert(i.ast_info().node_id());
            i.create_constraints(&mut state, ctx);
        }

        let variables = Self::get_variables(&state.constraints);

        for i in &state.constraints {
            println!("{}", i);
        }

        let solver = Solver::new(&variables, &state.constraints);
        solver.solve()?;

        Ok(())
    }

    fn get_variables(constraints: &[Constraint<TYPE, ERR>]) -> Vec<&Variable<TYPE, ERR>> {
        constraints
            .iter()
            .flat_map(|i| i.variables())
            .dedup_by(|i, j| i.id() == j.id())
            .collect()
    }
}
