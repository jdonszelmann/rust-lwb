use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use crate::codegen_prelude::{AstInfo, AstNode};
use crate::parser::ast::NodeId;
use crate::typechecker::constraints::{Constraint, FreeVariable, KnownVariable, new_variable_id, Variable, VariableId};
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::{Type, TypeCheckable};

pub struct State<M: AstInfo, CTX, TYPE: Type> {
    node_type_vars: HashMap<NodeId, VariableId>,

    constraints: Vec<Constraint<TYPE>>,

    todo: VecDeque<Box<dyn TypeCheckable<M, CTX, TYPE>>>
}

impl<M: AstInfo, CTX, TYPE: Type> Default for State<M, CTX, TYPE> {
    fn default() -> Self {
        Self {
            node_type_vars: Default::default(),
            constraints: vec![],
            todo: Default::default()
        }
    }
}

impl<M: AstInfo, CTX, TYPE: Type> State<M, CTX, TYPE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_variable(&mut self) -> Variable<TYPE> {
        Free(Rc::new(FreeVariable {
            id: new_variable_id(),
        }))
    }

    // pub fn own_type_variable(&mut self, node: impl AstNode<M>) -> Variable<TYPE> {
    //     let node_id = node.ast_info().node_id();
    //     match self.node_type_vars.entry(node_id) {
    //         Vacant(e) => {
    //             self.counter += 1;
    //             e.insert(VariableId(self.counter));
    //             Free(FreeVariable {
    //                 id: VariableId(self.counter),
    //             })
    //         },
    //         Occupied(e) => {
    //             let id = e.get();
    //             Free(FreeVariable { id: *id })
    //         }
    //     }
    // }

    // pub fn value_of_type(&mut self, value: TYPE) -> Variable<TYPE> {
    //     self.counter = &self.counter + 1;
    //     Known(KnownVariable {
    //         id: VariableId(self.counter),
    //         value,
    //     })
    // }

    pub fn type_ok<T>(&mut self, ast_node: &T)
        where T: TypeCheckable<M, CTX, TYPE> {

    }

    pub fn get_type<T>(&mut self, ast_node: &T) -> Variable<TYPE>
        where T: TypeCheckable<M, CTX, TYPE> {

        todo!()
    }

    pub fn add_constraint(&mut self, constraint: Constraint<TYPE>) {

    }

    pub fn type_of_self<T>(&self, ast_node: &T) -> Variable<TYPE>
        where T: TypeCheckable<M, CTX, TYPE> {

        todo!()
    }
}
