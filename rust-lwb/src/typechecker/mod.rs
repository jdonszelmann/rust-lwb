use crate::parser::ast::{AstNode, SpannedAstInfo};
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::error::TypeError;
use crate::typechecker::solver::Solver;
use crate::typechecker::state::State;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod constraints;
pub mod error;
mod solver;
pub mod state;
mod union_find;

pub trait Type: Debug + PartialEq {}

pub trait TypeCheckable<M: SpannedAstInfo, CTX, TYPE: Type>: AstNode<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, CTX, TYPE>, ctx: &CTX);
}

impl<M: SpannedAstInfo, CTX, TYPE: Type, T> TypeCheckable<M, CTX, TYPE> for Box<T>
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

impl<M: SpannedAstInfo, CTX, TYPE: Type> TypeChecker<M, CTX, TYPE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn check_types<Ast>(self, ast: Ast, ctx: &CTX) -> Result<(), TypeError<TYPE>>
    where
        Ast: TypeCheckable<M, CTX, TYPE>,
    {
        let mut state = State::new();
        state.type_ok(&ast);
        state.current_depth = 1;

        let mut had = HashSet::new();

        while let Some((depth, i)) = state.todo.pop_front() {
            let nodeid = i.ast_info().node_id();
            if had.contains(&nodeid) {
                continue;
            }

            state.current_depth = depth + 1;
            had.insert(i.ast_info().node_id());
            i.create_constraints(&mut state, ctx);
        }

        let variables = Self::get_variables(state.constraints.iter().map(|i| &i.constraint));

        // for i in &state.constraints {
        //     println!("{:?}", i);
        // }

        let solver = Solver::new(&variables, &state.constraints);
        solver.solve()?;

        Ok(())
    }

    fn get_variables<'a>(
        constraints: impl IntoIterator<Item = &'a Constraint<TYPE>>,
    ) -> Vec<&'a Variable<TYPE>> {
        constraints
            .into_iter()
            .flat_map(|i| i.variables())
            .collect()
    }
}
