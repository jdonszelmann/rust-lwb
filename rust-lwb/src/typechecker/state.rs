use std::cmp::Ordering;
use crate::parser::ast::SpannedAstInfo;
use crate::sources::span::Span;
use crate::typechecker::constraints::{Constraint, Variable};
use crate::typechecker::{Type, TypeCheckable};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct OrderedConstraint<TYPE: Type> {
    pub(crate) depth: usize,
    pub(crate) constraint: Constraint<TYPE>
}

impl<TYPE: Type> Eq for OrderedConstraint<TYPE> {}

impl<TYPE: Type> PartialOrd for OrderedConstraint<TYPE> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.depth.partial_cmp(&other.depth)
    }
}

impl<TYPE: Type> Ord for OrderedConstraint<TYPE> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth)
    }
}

impl<TYPE: Type> PartialEq for OrderedConstraint<TYPE> {
    fn eq(&self, other: &Self) -> bool {
        self.depth.eq(&other.depth)
    }
}


impl<TYPE: Type> OrderedConstraint<TYPE> {
    pub fn new(constraint: Constraint<TYPE>, depth: usize) -> Self {
        Self {
            depth,
            constraint
        }
    }
}


pub struct State<'ast, M: SpannedAstInfo, CTX, TYPE: Type> {
    pub(crate) constraints: Vec<OrderedConstraint<TYPE>>,

    pub(crate) todo: VecDeque<(usize, &'ast dyn TypeCheckable<M, CTX, TYPE>)>,
    pub(crate) current_depth: usize,
}

impl<'ast, M: SpannedAstInfo, CTX, TYPE: Type> Default for State<'ast, M, CTX, TYPE> {
    fn default() -> Self {
        Self {
            constraints: vec![],
            todo: Default::default(),
            current_depth: 0
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
        self.todo.push_back((self.current_depth, ast_node));
    }

    pub fn get_type<T>(&mut self, ast_node: &'ast T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        self.type_ok(ast_node);
        self.type_of_node(ast_node)
    }

    pub fn add_constraint(&mut self, constraint: Constraint<TYPE>) {
        self.constraints.push(OrderedConstraint::new(constraint, self.current_depth));
    }

    fn type_of_node<T>(&mut self, ast_node: &T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        let node_var = self.new_variable_with_span(ast_node.ast_info().span().clone());

        self.add_constraint(Constraint::Node(node_var.clone(), ast_node.ast_info().node_id()));

        node_var
    }

    pub fn type_of_self<T>(&mut self, ast_node: &T) -> Variable<TYPE>
    where
        T: TypeCheckable<M, CTX, TYPE>,
    {
        self.type_of_node(ast_node)
    }
}
