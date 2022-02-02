use crate::codegen_prelude::AstInfo;
use crate::parser::ast::{AstNode, NodeId};
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::Type;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Copy, Clone)]
pub struct VariableId(usize);

pub struct KnownVariable<TYPE> {
    id: VariableId,
    value: TYPE,
}

pub struct FreeVariable {
    id: VariableId,
}

impl FreeVariable {
    fn make_known<TYPE>(&self, found_type: TYPE) -> KnownVariable<TYPE> {
        KnownVariable {
            id: self.id,
            value: found_type,
        }
    }
}

pub enum Variable<TYPE> {
    Free(FreeVariable),
    Known(KnownVariable<TYPE>),
}

pub trait ComputedConstraint<TYPE: Type> {
    fn solve(&self, input: &[KnownVariable<TYPE>]) -> Constraint<TYPE>;
}

impl<TYPE: Type, F> ComputedConstraint<TYPE> for F
where
    F: Fn(&[KnownVariable<TYPE>]) -> Constraint<TYPE>,
{
    fn solve(&self, input: &[KnownVariable<TYPE>]) -> Constraint<TYPE> {
        (self)(input)
    }
}

pub enum Constraint<TYPE: Type> {
    And(Box<Constraint<TYPE>>, Box<Constraint<TYPE>>),
    Or(Box<Constraint<TYPE>>, Box<Constraint<TYPE>>),
    Not(Box<Constraint<TYPE>>),

    Eq(Variable<TYPE>, Variable<TYPE>),
    Node(Variable<TYPE>, NodeId),

    Computed(Box<dyn ComputedConstraint<TYPE>>),
}

impl<TYPE: Type> Variable<TYPE> {
    pub fn eq(self, other: Variable<TYPE>) -> Constraint<TYPE> {
        Constraint::Eq(self, other)
    }

    pub fn is_free(&self) -> bool {
        match self {
            Variable::Free(_) => true,
            Variable::Known(_) => false,
        }
    }
}

impl<TYPE: Type> Constraint<TYPE> {
    pub fn and(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
        Constraint::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
        Constraint::Or(Box::new(self), Box::new(other))
    }

    pub fn not(self) -> Constraint<TYPE> {
        Not::not(self)
    }
}

impl<TYPE: Type> BitAnd for Constraint<TYPE> {
    type Output = Constraint<TYPE>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl<TYPE: Type> BitOr for Constraint<TYPE> {
    type Output = Constraint<TYPE>;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

impl<TYPE: Type> Not for Constraint<TYPE> {
    type Output = Constraint<TYPE>;

    fn not(self) -> Self::Output {
        Constraint::Not(Box::new(self))
    }
}

pub fn not<TYPE: Type>(c1: Constraint<TYPE>) -> Constraint<TYPE> {
    c1.not()
}

#[derive(Default)]
pub struct ConstraintBuilder {
    counter: usize,
    node_type_vars: HashMap<NodeId, VariableId>,
}

impl ConstraintBuilder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ConstraintBuilder {
    pub fn free_variable<TYPE>(&mut self) -> Variable<TYPE> {
        self.counter += 1;
        Free(FreeVariable {
            id: VariableId(self.counter),
        })
    }

    pub fn own_type_variable<M: AstInfo, TYPE>(&mut self, node: impl AstNode<M>) -> Variable<TYPE> {
        let node_id = node.ast_info().node_id();
        match self.node_type_vars.entry(node_id) {
            Vacant(e) => {
                self.counter += 1;
                e.insert(VariableId(self.counter));
                Free(FreeVariable {
                    id: VariableId(self.counter),
                })
            }
            Occupied(e) => {
                let id = e.get();
                Free(FreeVariable { id: *id })
            }
        }
    }

    pub fn value_of_type<TYPE>(&mut self, value: TYPE) -> Variable<TYPE> {
        self.counter = &self.counter + 1;
        Known(KnownVariable {
            id: VariableId(self.counter),
            value,
        })
    }
}
