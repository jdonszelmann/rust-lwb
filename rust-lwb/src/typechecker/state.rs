use crate::parser::ast::SpannedAstInfo;
use crate::sources::span::Span;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::{Type, TypeCheckable};
use std::collections::VecDeque;

pub struct State<'ast, M: SpannedAstInfo, CTX, TYPE: Type> {
    pub(crate) constraints: Vec<Constraint<TYPE>>,

    pub(crate) todo: VecDeque<&'ast dyn TypeCheckable<M, CTX, TYPE>>,
}

impl<'ast, M: SpannedAstInfo, CTX, TYPE: Type> Default for State<'ast, M, CTX, TYPE> {
    fn default() -> Self {
        Self {
            constraints: vec![],
            todo: Default::default(),
        }
    }
}

impl<'ast, M: SpannedAstInfo, CTX, TYPE: Type> State<'ast, M, CTX, TYPE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_variable(&self) -> Variable<TYPE> {
        Variable::new_free(None)
    }

    pub(crate) fn new_variable_with_span(&self, span: Span) -> Variable<TYPE> {
        Variable::new_free(Some(span))
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
        let var = self.new_variable_with_span(ast_node.ast_info().span().clone());

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
