use crate::parser::ast::NodeId;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::error::TypeError;
use crate::typechecker::union_find::UnionFind;
use crate::typechecker::Type;
use std::collections::HashMap;

pub struct Solver<'var, TYPE: Type, ERR> {
    constraints: &'var [Constraint<TYPE, ERR>],
    nodes: HashMap<NodeId, &'var Variable<TYPE, ERR>>,
    uf: UnionFind<'var, TYPE, ERR>,
}

impl<'var, TYPE: Type, ERR> Solver<'var, TYPE, ERR> {
    pub fn new(
        variables: &[&'var Variable<TYPE, ERR>],
        constraints: &'var [Constraint<TYPE, ERR>],
    ) -> Self {
        let mut uf = UnionFind::new();

        for &i in variables {
            uf.insert(i);
        }

        Self {
            constraints,
            nodes: Default::default(),
            uf,
        }
    }

    fn solve_constraint(
        &mut self,
        constraint: &'var Constraint<TYPE, ERR>,
    ) -> Result<(), TypeError<TYPE, ERR>> {
        println!("solving {}", constraint);
        match constraint {
            Constraint::And(a, b) => {
                self.solve_constraint(a)?;
                self.solve_constraint(b)?;
            }
            Constraint::Equiv(a, b) => {
                self.uf.union(a, b)?;
            }
            Constraint::NotEquiv(_, _) => {
                todo!()
            }
            Constraint::Node(var, node) => {
                let nodevar = *self.nodes.entry(*node).or_insert(var);
                self.uf.union(nodevar, var)?;
            }
            Constraint::Computed(_) => {
                todo!()
            }
        }

        println!("{:?}", self.uf);

        Ok(())
    }

    pub fn solve(mut self) -> Result<(), TypeError<TYPE, ERR>> {
        for i in self.constraints {
            self.solve_constraint(i)?
        }

        Ok(())
    }
}
