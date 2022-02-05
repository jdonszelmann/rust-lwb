use crate::parser::ast::NodeId;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::error::TypeError;
use crate::typechecker::union_find::UnionFind;
use crate::typechecker::Type;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Deref, DerefMut};
use crate::typechecker::state::OrderedConstraint;

pub struct Solver<'var, TYPE: Type> {
    constraints: &'var [OrderedConstraint<TYPE>],
    nodes: HashMap<NodeId, &'var Variable<TYPE>>,
    uf: UnionFind<'var, TYPE>,
}

// pub struct Ordered<T, O: Ord>(T, O);
//
// impl<T, O: Ord> DerefMut for Ordered<T, O> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl<T, O: Ord> Deref for Ordered<T, O> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl<T, O: Ord> Ordered<T, O> {
//     pub fn new(value: T, order: O) -> Self {
//         Self(value, order)
//     }
//
//     pub fn into_inner(self) -> T {
//         self.0
//     }
//
//     pub fn ordering(&self) -> &O {
//         &self.1
//     }
// }
//
// impl<T, O: Ord> Eq for Ordered<T, O> {}
//
// impl<T, O: Ord> PartialEq<Self> for Ordered<T, O> {
//     fn eq(&self, other: &Self) -> bool {
//         self.1.eq(&other.1)
//     }
// }
//
// impl<T, O: Ord> PartialOrd<Self> for Ordered<T, O> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         self.1.partial_cmp(&other.1)
//     }
// }
//
// impl<T, O: Ord> Ord for Ordered<T, O> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.1.cmp(&other.1)
//     }
// }

impl<'var, TYPE: Type> Solver<'var, TYPE> {
    pub fn new(variables: &[&'var Variable<TYPE>], constraints: &'var [OrderedConstraint<TYPE>]) -> Self {
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
        constraint: &'var Constraint<TYPE>,
    ) -> Result<(), TypeError<TYPE>> {
        match constraint {
            Constraint::And(a, b) => {
                self.solve_constraint(a)?;
                self.solve_constraint(b)?;
            }
            Constraint::Equiv(a, b) => {
                println!("solving {:?}", constraint);
                self.uf.union(a, b)?;
            }
            Constraint::NotEquiv(_, _) => {
                todo!()
            }
            Constraint::Node(var, node) => {
                let nodevar = *self.nodes.entry(*node).or_insert(var);
                println!("solving node:{:?} == {:?}", nodevar, var);
                self.uf.union(nodevar, var)?;
            }
            Constraint::Computed(_) => {
                todo!()
            }
        }

        println!("{:?}", self.uf);

        Ok(())
    }

    pub fn solve(mut self) -> Result<(), TypeError<TYPE>> {
        let mut queue = BinaryHeap::new();
        for i in self.constraints {
            queue.push(i);
        }

        while let Some(constraint) = queue.pop() {
            // we just peeked
            self.solve_constraint(&constraint.constraint)?;
        }
        Ok(())
    }
}
