use crate::codegen_prelude::AstInfo;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::{Type, TypeCheckable};
use std::collections::VecDeque;

pub struct State<'ast, M: AstInfo, CTX, TYPE: Type, ERR> {
    pub(crate) constraints: Vec<Constraint<TYPE, ERR>>,

    pub(crate) todo: VecDeque<&'ast dyn TypeCheckable<M, CTX, TYPE, ERR>>,
}

impl<'ast, M: AstInfo, CTX, TYPE: Type, ERR> Default for State<'ast, M, CTX, TYPE, ERR> {
    fn default() -> Self {
        Self {
            constraints: vec![],
            todo: Default::default(),
        }
    }
}

impl<'ast, M: AstInfo, CTX, TYPE: Type, ERR> State<'ast, M, CTX, TYPE, ERR> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_variable(&self) -> Variable<TYPE, ERR> {
        Variable::new_free()
    }

    pub fn type_ok<T>(&mut self, ast_node: &'ast T)
    where
        T: TypeCheckable<M, CTX, TYPE, ERR>,
    {
        self.todo.push_back(ast_node);
    }

    pub fn get_type<T>(&mut self, ast_node: &'ast T) -> Variable<TYPE, ERR>
    where
        T: TypeCheckable<M, CTX, TYPE, ERR>,
    {
        self.type_ok(ast_node);
        self.type_of_node(ast_node)
    }

    pub fn add_constraint(&mut self, constraint: Constraint<TYPE, ERR>) {
        self.constraints.push(constraint);
    }

    fn type_of_node<T>(&mut self, ast_node: &T) -> Variable<TYPE, ERR>
    where
        T: TypeCheckable<M, CTX, TYPE, ERR>,
    {
        let var = self.new_variable();

        self.add_constraint(Constraint::Node(var.clone(), ast_node.ast_info().node_id()));

        var
    }

    pub fn type_of_self<T>(&mut self, ast_node: &T) -> Variable<TYPE, ERR>
    where
        T: TypeCheckable<M, CTX, TYPE, ERR>,
    {
        self.type_of_node(ast_node)
    }
}
