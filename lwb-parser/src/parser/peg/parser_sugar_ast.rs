use crate::sources::character_class::CharacterClass;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constructor {
    pub documentation: Option<String>,
    pub name: String,
    pub expression: Expression,
    pub annotations: Vec<Annotation>,
    /// This is set when rules are merged with part-of. In that case,
    /// constructors that used to link the two rules should not be put in the ast.
    pub dont_put_in_ast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expression {
    Sort(String),
    Literal(String),
    Sequence(Vec<Expression>),
    Repeat {
        e: Box<Expression>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Expression>),
    Delimited {
        e: Box<Expression>,
        delim: Box<Expression>,
        min: u64,
        max: Option<u64>,
        trailing: bool,
    },

    Negative(Box<Expression>),
    Positive(Box<Expression>),
}

#[derive(Debug, Clone, Display, Serialize, Deserialize, PartialEq, Eq)]
pub enum Annotation {
    #[display(fmt = "no-pretty-print")]
    NoPrettyPrint,

    /// Don't accept layout in this rule and any child rule
    #[display(fmt = "no-layout")]
    NoLayout,

    #[display(fmt = "injection")]
    Injection,

    /// represent this constructor as a single string in the final ast
    #[display(fmt = "single-string")]
    SingleString,

    /// this rule does not appear in the ast anywhere its used.
    #[display(fmt = "hidden")]
    Hidden,

    /// This rule gives an error when parsed
    #[display(fmt = "error")]
    Error(String),

    /// Says that one rule must generate its constructors as part of another rule
    #[display(fmt = "part-of: {}", _0)]
    PartOf(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub documentation: Option<String>,
    pub name: String,
    pub constructors: Vec<Constructor>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyntaxFileAst {
    pub sorts: HashMap<String, Sort>,
    pub starting_sort: String,
    pub merges: HashMap<String, String>,
    pub old_sort_names: Vec<String>,
}

#[derive(Error, Debug, Clone)]
pub enum SimplifyError {
    #[error("your `part-of` annotations form a cycle")]
    Cycle,

    #[error("You marked {0} as part of {1} but {1} contains no rule to get to {0} (a line in the form of `{0};`")]
    NoConnection(String, String),

    #[error("You marked the starting sort ({0}) as part of another sort ({1}). This is not allowed.")]
    StartingSort(String, String),

    #[error("The AST was simplified twice. This is a bug.")]
    AlreadySimplified
}

impl SyntaxFileAst {
    pub fn names(&self, name: &str) -> Vec<&str> {
        let mut result = Vec::new();

        for i in &self.old_sort_names {
            let new_name = Self::get_new_name(i, &self.merges);
            if new_name == name {
                result.push(i.as_str());
            }
        }

        result
    }


    /// Simplification of the ast means that all rules that are marked as `part-of` are
    /// actually integrated with their associated sort. This is useful before code generation
    /// since in codegen they actually are one sort.
    pub fn simplify(mut self) -> Result<Self, SimplifyError> {
        if !self.merges.is_empty() {
            return Err(SimplifyError::AlreadySimplified);
        }


        let mut order = Vec::new();
        let mut had = HashSet::new();
        let mut todo = Vec::new();

        let old_sort_names = self.sorts.keys().map(|i| i.clone()).collect::<Vec<_>>();

        // determine all relations between sorts.
        // handle the ones that aren't part-of another sort last
        // by pretty much topologically sorting them.
        for (name, sort) in self.sorts {
            if let Some(other) = sort.annotations.iter()
                .find_map(|i| {
                    if let Annotation::PartOf(a) = i {
                        Some(a)
                    } else {
                        None
                    }
                })
            {
                let other = other.clone();
                todo.push((name, sort, other));
            } else {
                had.insert(name.clone());
                order.push((name, sort, None));
            }
        }

        // for the ones that had a part-of relation, do topological sort
        // and add then to the ordering
        while !todo.is_empty() {
            let start_length = todo.len();
            let mut new_todo = Vec::new();

            for (name, sort, part_of) in todo {
                if had.contains(&part_of) {
                    had.insert(name.clone());
                    order.push((name, sort, Some(part_of)));
                } else {
                    new_todo.push((name, sort, part_of));
                }
            }

            if start_length == new_todo.len() {
                return Err(SimplifyError::Cycle)
            }

            todo = new_todo;
        }

        // keep some refs to the ordered sorts
        let sort_refs: HashMap<_, _> = order.iter().map(|(name, sort, _)| (name, sort)).collect();

        // now determine if this set of relations is allowed. Are there proper paths between
        // related sorts?
        for (name, sort, part_of) in &order {
            if let Some(other) = part_of {
                if sort.name == self.starting_sort {
                    return Err(SimplifyError::StartingSort(name.clone(), other.to_string()));
                }
            }

            if let Some(other) = part_of.as_ref().map(|i| sort_refs.get(i)).flatten() {
                let mut has_connection = false;
                for i in &other.constructors {
                    if i.expression == Expression::Sort(sort.name.clone()) {
                        has_connection = true;
                        break;
                    }
                }

                if !has_connection {
                    return Err(SimplifyError::NoConnection(sort.name.clone(), other.name.clone()))
                }
            }
        }

        // now generate some new sorts
        let mut new_sorts = HashMap::new();
        let mut merges = HashMap::<_, Vec<Sort>>::new();
        let mut occurred_merges = HashMap::new();

        // evaluate the sorts in the predetermined order (but in reverse)
        for (name, mut sort, part_of) in order.into_iter().rev() {
            // if another sort needs to merge into this one? Do so.
            if let Some(others) = merges.remove(&name) {
                for other in others {
                    // extend the constructors
                    sort.constructors = sort.constructors.into_iter()
                        .map(|mut i| {
                            i.dont_put_in_ast = i.expression == Expression::Sort(other.name.clone());
                            i
                        })
                        .chain(other.constructors)
                        .collect();


                    // merge documentation nicely
                    if let Some(ref documentation) = other.documentation {
                        if let Some(ref mut i) = sort.documentation {
                            *i = format!("{i}\n\n# {name}\n since {name} is a part of {}, its documentation is shown below:\n{documentation}", sort.name);
                        } else {
                            sort.documentation = Some(format!("(from {name}:)\n{documentation}"));
                        }
                    }

                    occurred_merges.insert(other.name, name.clone());
                }
            }

            // some sorts should be merged with others. Mark these in `merges`
            if let Some(part_of) = part_of {
                merges.entry(part_of).or_insert(vec![]).push(sort);
            } else {
                // when we found a sort that needs no more merging, add them to the new list
                // of sorts
                new_sorts.insert(name, sort);
            }
        }

        // now some sorts got removed. Rename their references to the appropriate parent.
        // then undo partial move (and restore self)
        self.sorts = new_sorts.into_iter().map(|(name, mut sort)| {
            sort.constructors = sort.constructors.into_iter()
                .map(|mut c| {
                    c.expression = Self::rewrite_expression(c.expression, &occurred_merges);
                    c
                }).collect();

            (name, sort)
        }).collect();

        self.merges = occurred_merges;
        self.old_sort_names = old_sort_names;

        Ok(self)
    }

    fn get_new_name(name: &String, merges: &HashMap<String, String>) -> String {
        if let Some(new_name) = merges.get(name) {
            Self::get_new_name(new_name, merges)
        } else {
            name.clone()
        }
    }

    fn rewrite_expression(e: Expression, merges: &HashMap<String, String>) -> Expression {
        match e {
            Expression::Sort(name) => {
                Expression::Sort(Self::get_new_name(&name, merges))
            }
            a@Expression::Literal(_) => a,
            Expression::Sequence(s) => Expression::Sequence(
                s.into_iter()
                    .map(|e| Self::rewrite_expression(e, merges))
                    .collect()
            ),
            Expression::Repeat { e, min, max } => Expression::Repeat {
                e: Box::new(Self::rewrite_expression(*e, merges)),
                min,
                max
            },
            a@Expression::CharacterClass(_) => a,
            Expression::Choice(s) => Expression::Choice(
                s.into_iter()
                    .map(|e| Self::rewrite_expression(e, merges))
                    .collect()
            ),
            Expression::Delimited { e, delim, min, max, trailing } => Expression::Delimited {
                e: Box::new(Self::rewrite_expression(*e, merges)),
                delim,
                min,
                max,
                trailing
            },
            Expression::Negative(e) => Expression::Negative(Box::new(Self::rewrite_expression(*e, merges))),
            Expression::Positive(e) => Expression::Positive(Box::new(Self::rewrite_expression(*e, merges))),
        }
    }
}
