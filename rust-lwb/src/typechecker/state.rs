use crate::codegen_prelude::AstInfo;
use crate::typechecker::constraints::Variable::Free;
use crate::typechecker::constraints::{new_variable_id, Constraint, FreeVariable, Variable};
use crate::typechecker::{Type, TypeCheckable};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct State<'ast, M: AstInfo, CTX, TYPE: Type> {
    constraints: Vec<Constraint<TYPE>>,

    todo: VecDeque<&'ast dyn TypeCheckable<M, CTX, TYPE>>,
}

impl<'ast, M: AstInfo, CTX, TYPE: Type> Default for State<'ast, M, CTX, TYPE> {
    fn default() -> Self {
        Self {
            constraints: vec![],
            todo: Default::default(),
        }
    }
}

impl<'ast, M: AstInfo, CTX, TYPE: Type> State<'ast, M, CTX, TYPE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_variable(&self) -> Variable<TYPE> {
        Free(Rc::new(FreeVariable {
            id: new_variable_id(),
        }))
    }

    pub(crate) fn next_node_to_typecheck(
        &mut self,
    ) -> Option<&'ast dyn TypeCheckable<M, CTX, TYPE>> {
        self.todo.pop_front()
    }

    pub fn type_ok<T>(&mut self, ast_node: &'ast T)
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        self.todo.push_back(ast_node);
    }

    pub fn get_type<T>(&mut self, ast_node: &'ast T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        self.type_ok(ast_node);
        self.type_of_node(ast_node)
    }

    pub fn add_constraint(&mut self, constraint: Constraint<TYPE>) {
        self.constraints.push(constraint);
    }

    fn type_of_node<T>(&mut self, ast_node: &T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        let var = self.new_variable();

        self.add_constraint(Constraint::Node(var.clone(), ast_node.ast_info().node_id()));

        var
    }

    pub fn type_of_self<T>(&mut self, ast_node: &T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        self.type_of_node(ast_node)
    }
}
