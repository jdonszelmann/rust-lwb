use crate::codegen_prelude::AstInfo;
use crate::parser::ast::{AstNode, NodeId};
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::Type;
use std::collections::HashMap;

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
    fn solve(&self, input: &Vec<KnownVariable<TYPE>>) -> Constraint<TYPE>;
}

impl<TYPE: Type, F> ComputedConstraint<TYPE> for F
where
    F: Fn(&Vec<KnownVariable<TYPE>>) -> Constraint<TYPE>,
{
    fn solve(&self, input: &Vec<KnownVariable<TYPE>>) -> Constraint<TYPE> {
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
        Constraint::Not(Box::new(self))
    }
}

pub fn not<TYPE: Type>(c1: Constraint<TYPE>) -> Constraint<TYPE> {
    c1.not()
}

pub struct ConstraintBuilder {
    counter: usize,
    node_type_vars: HashMap<NodeId, VariableId>,
}

impl ConstraintBuilder {
    pub fn new() -> Self {
        ConstraintBuilder {
            counter: 0,
            node_type_vars: HashMap::new(),
        }
    }
}

impl ConstraintBuilder {
    pub fn free_variable<TYPE>(&mut self) -> Variable<TYPE> {
        self.counter = &self.counter + 1;
        Free(FreeVariable {
            id: VariableId(self.counter),
        })
    }

    pub fn own_type_variable<M: AstInfo, TYPE>(&mut self, node: impl AstNode<M>) -> Variable<TYPE> {
        let node_id = node.ast_info().node_id();
        if self.node_type_vars.contains_key(&node_id) {
            let id = self.node_type_vars.get(&node_id).unwrap();
            Free(FreeVariable { id: *id })
        } else {
            self.counter = &self.counter + 1;
            self.node_type_vars
                .insert(node_id, VariableId(self.counter));
            Free(FreeVariable {
                id: VariableId(self.counter),
            })
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
