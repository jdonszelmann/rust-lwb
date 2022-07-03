use crate::parser::peg::parser_sugar_ast::{Expression, Sort, SyntaxFileAst};
use by_address::ByAddress;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashSet, VecDeque};
use typed_arena::Arena;

type Todo<'ast, 'arena> = VecDeque<(&'ast Sort, &'arena RecursionChecker<'ast, 'arena>)>;

/// Iterates in a breadth first order through all the sorts
/// in the AST. Also keeps track of cycles so recursive types can
/// be found.
///
/// Usage (you need to make an arena for it to allocate in):
/// ```no_run, compile_fail
/// let arena = Default::default();
/// let sorts_iterator = BreadthFirstAstIterator::new(syntax, &arena);
/// ```
pub struct BreadthFirstAstIterator<'ast, 'arena> {
    ast: &'ast SyntaxFileAst,
    arena: &'arena Arena<RecursionChecker<'ast, 'arena>>,
    had: HashSet<ByAddress<&'ast Sort>>,
    todo: Todo<'ast, 'arena>,
}

impl<'ast, 'arena> BreadthFirstAstIterator<'ast, 'arena> {
    pub fn new(
        ast: &'ast SyntaxFileAst,
        arena: &'arena Arena<RecursionChecker<'ast, 'arena>>,
    ) -> Self {
        // find either the starting sort, or any random sort if there is no starting sort.
        let todo = ast
            .sorts
            .get(&ast.starting_sort)
            .or_else(|| ast.sorts.values().next())
            .iter()
            .copied()
            .map(|i| (i, &*arena.alloc(RecursionChecker::Root.child(&i.name))))
            .collect::<Todo>();

        Self {
            ast,
            arena,
            had: Default::default(),
            todo,
        }
    }

    /// Recursively traverses an expression to find sorts that it references.
    /// This is necessary to do the depth first traversal of the ast.
    fn find_referenced_sorts(
        &mut self,
        exp: &'ast Expression,
        ckr: &'arena RecursionChecker<'ast, 'arena>,
    ) {
        match exp {
            // if its a sort, add it to the queue
            Expression::Sort(s) => {
                if let Some(i) = self.ast.sorts.get(s.as_str()) {
                    if !self.had.contains(&ByAddress(i)) {
                        self.todo.push_back((i, &*self.arena.alloc(ckr.child(s))));
                    }
                }
            }
            // otherwise just recursively check subexpressions
            Expression::Sequence(s) | Expression::Choice(s) => {
                s.iter().for_each(|i| self.find_referenced_sorts(i, ckr))
            }
            Expression::Repeat { e, .. } | Expression::Delimited { e, .. } => {
                self.find_referenced_sorts(e, ckr)
            }
            Expression::Negative(_) => todo!(),
            Expression::Positive(_) => todo!(),
            _ => {}
        }
    }

    /// Find a sort in the ast, that we haven't yet traversed
    pub fn find_not_had(&self) -> Option<&'ast Sort> {
        self.ast
            .sorts
            .values()
            .find(|&s| !self.had.contains(&ByAddress(s)))
    }
}

impl<'ast, 'arena> Iterator for BreadthFirstAstIterator<'ast, 'arena> {
    type Item = (&'ast Sort, &'arena RecursionChecker<'ast, 'arena>);

    fn next(&mut self) -> Option<Self::Item> {
        // is there a sort in the queue?
        if let Some((sort, ckr)) = self.todo.pop_front() {
            if self.had.contains(&ByAddress(sort)) {
                return self.next();
            }

            // now we've had it
            self.had.insert(ByAddress(sort));

            // find all referenced sorts and put them in the queue
            for i in &sort.constructors {
                self.find_referenced_sorts(&i.expression, ckr);
            }

            // return this one
            Some((sort, ckr))

            // have we now had all of them?
        } else if self.ast.sorts.len() == self.had.len() {
            None

            // no? then find one we haven't yet had (because it was "dead code" in the AST)
            // then just restart the algorithm there
        } else {
            let not_had = self.find_not_had()?;
            self.todo.push_back((
                not_had,
                &*self
                    .arena
                    .alloc(RecursionChecker::Root.child(&not_had.name)),
            ));
            self.next()
        }
    }
}

// linked list that keeps track of which types we've seen during our search through sorts
pub enum RecursionChecker<'s, 'par> {
    Root,
    Child {
        parent: &'par RecursionChecker<'s, 'par>,
        name: &'s str,
    },
}

impl Default for RecursionChecker<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, 'par> RecursionChecker<'s, 'par> {
    pub fn new() -> Self {
        Self::Root
    }

    pub fn child(&'par self, name: &'s str) -> RecursionChecker<'s, 'par> {
        Self::Child { parent: self, name }
    }

    pub fn maybe_box(&self, generated_type: &str, ts: TokenStream) -> TokenStream {
        if self.needs_box(generated_type) {
            quote!(Box::new(#ts))
        } else {
            ts
        }
    }

    pub fn maybe_box_type(&self, generated_type: &str, ts: TokenStream) -> TokenStream {
        if self.needs_box(generated_type) {
            quote!(Box<#ts>)
        } else {
            ts
        }
    }

    pub fn needs_box(&self, generated_type: &str) -> bool {
        match self {
            Self::Root => false,
            Self::Child { name, .. } if *name == generated_type => true,
            Self::Child { parent, .. } => parent.needs_box(generated_type),
        }
    }
}
