use crate::codegen_prelude::AstInfo;
use crate::parser::ast::NodeId;
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::state::State;
use crate::typechecker::Type;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::BitAnd;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBALLY_UNIQUE_VARIABLE_ID: AtomicU64 = AtomicU64::new(0);
pub fn new_variable_id() -> VariableId {
    let value = GLOBALLY_UNIQUE_VARIABLE_ID.fetch_add(1, Ordering::Relaxed);
    VariableId(value)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct VariableId(u64);

impl Display for VariableId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct KnownVariable<TYPE: Type, ERR> {
    id: VariableId,
    pub(crate) value: TYPE,
    phantom: PhantomData<ERR>,
}

#[derive(Debug)]
pub struct FreeVariable<ERR> {
    id: VariableId,
    phantom: PhantomData<ERR>,
}

#[derive(Debug)]
pub enum Variable<TYPE: Type, ERR> {
    Free(Rc<FreeVariable<ERR>>),
    Known(Rc<KnownVariable<TYPE, ERR>>),
}

impl<TYPE: Type, ERR> Display for Variable<TYPE, ERR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Free(fv) => write!(f, "var({})", fv.id),
            Known(k) => write!(f, "var({:?}, {})", k.value, k.id),
        }
    }
}

impl<TYPE: Type, ERR> Variable<TYPE, ERR> {
    pub(crate) fn new_free() -> Self {
        Self::Free(Rc::new(FreeVariable {
            id: new_variable_id(),
            phantom: Default::default(),
        }))
    }

    pub(crate) fn new_known(value: TYPE) -> Self {
        Self::Known(Rc::new(KnownVariable {
            id: new_variable_id(),
            value,
            phantom: Default::default(),
        }))
    }

    pub(crate) fn id(&self) -> VariableId {
        match self {
            Free(i) => i.id,
            Known(i) => i.id,
        }
    }
}

impl<TYPE: Type, ERR> Clone for Variable<TYPE, ERR> {
    fn clone(&self) -> Self {
        match self {
            Free(v) => Free(Rc::clone(v)),
            Known(v) => Known(Rc::clone(v)),
        }
    }
}

#[doc(hidden)]
mod sealed {
    use crate::typechecker::constraints::Variable;
    use crate::typechecker::Type;

    pub trait Sealed {}
    impl<TYPE: Type, ERR> Sealed for Variable<TYPE, ERR> {}
    impl<TYPE: Type, ERR> Sealed for &Variable<TYPE, ERR> {}
    impl<TYPE: Type> Sealed for TYPE {}
}

/// Converts things into variables. Trivially, variables
/// are convertible into variables. But types also are.
/// This trait can be seen in [`Variable::equiv`]
pub trait IntoVariable<TYPE: Type, ERR>: sealed::Sealed {
    fn into(self) -> Variable<TYPE, ERR>;
}

impl<TYPE: Type, ERR> IntoVariable<TYPE, ERR> for TYPE {
    fn into(self) -> Variable<TYPE, ERR> {
        Variable::new_known(self)
    }
}

impl<TYPE: Type, ERR> IntoVariable<TYPE, ERR> for Variable<TYPE, ERR> {
    fn into(self) -> Variable<TYPE, ERR> {
        self
    }
}

impl<TYPE: Type, ERR> IntoVariable<TYPE, ERR> for &Variable<TYPE, ERR> {
    fn into(self) -> Variable<TYPE, ERR> {
        self.clone()
    }
}

pub struct ComputedConstraint<TYPE: Type, ERR> {
    depends_on: Vec<Variable<TYPE, ERR>>,
    #[allow(dead_code)]
    compute: Box<dyn Fn(&[TYPE]) -> Result<TYPE, ERR>>,
}

pub enum Constraint<TYPE: Type, ERR> {
    And(Box<Self>, Box<Self>),

    // TODO: if you ever want to build a csp solver, feel free to uncomment these
    // Or(Box<Self>, Box<Self>),
    // Not(Box<Self>),
    Equiv(Variable<TYPE, ERR>, Variable<TYPE, ERR>),
    NotEquiv(Variable<TYPE, ERR>, Variable<TYPE, ERR>),
    Node(Variable<TYPE, ERR>, NodeId),

    Computed(ComputedConstraint<TYPE, ERR>),
}

impl<TYPE: Type, ERR> Display for Constraint<TYPE, ERR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::And(a, b) => write!(f, "{} & {}", a, b),
            Constraint::Equiv(a, b) => write!(f, "{} == {}", a, b),
            Constraint::NotEquiv(a, b) => write!(f, "{} != {}", a, b),
            Constraint::Node(var, nodeid) => {
                write!(f, "{:?} == {}", nodeid, var)
            }
            Constraint::Computed(ComputedConstraint { depends_on, .. }) => {
                write!(
                    f,
                    "computed constraint depending on [{}]",
                    depends_on.iter().map(|i| i.to_string()).join(",")
                )
            }
        }
    }
}

impl<TYPE: Type, ERR> Constraint<TYPE, ERR> {
    pub fn variables(&self) -> Vec<&Variable<TYPE, ERR>> {
        match self {
            Constraint::And(a, b) => {
                let mut variables = a.variables();
                variables.extend(b.variables());
                variables
            }
            Constraint::NotEquiv(a, b) => vec![a, b],
            Constraint::Equiv(a, b) => vec![a, b],
            Constraint::Node(a, _n) => vec![a],
            Constraint::Computed(ComputedConstraint { depends_on, .. }) => {
                depends_on.iter().collect()
            }
        }
    }
}

impl<TYPE: Type, ERR> Variable<TYPE, ERR> {
    /// Places a constraint on two variables, namely that they are equivalent
    /// and that their types should be the same.
    ///
    /// Can either accept a variable:
    /// ```rust
    /// // TODO
    /// ```
    ///
    /// Or a type, in which case it asserts that a variable is equal to a specific type:
    /// ```rust
    /// // TODO
    /// ```
    #[must_use]
    pub fn equiv(&self, other: impl IntoVariable<TYPE, ERR>) -> Constraint<TYPE, ERR> {
        Constraint::Equiv(self.clone(), other.into())
    }

    #[must_use]
    pub fn not_equiv(&self, other: impl IntoVariable<TYPE, ERR>) -> Constraint<TYPE, ERR> {
        Constraint::NotEquiv(self.clone(), other.into())
    }

    #[must_use]
    pub fn all_equiv<const N: usize>(
        &self,
        variables: [Variable<TYPE, ERR>; N],
    ) -> Constraint<TYPE, ERR> {
        assert!(variables.len() >= 2);

        let mut res: Option<Constraint<TYPE, ERR>> = None;

        for i in variables.windows(2) {
            if let Some(r) = res {
                res = Some(r.and(i[0].equiv(&i[1])));
            } else {
                res = Some(i[0].equiv(&i[1]));
            }
        }

        res.unwrap()
    }

    #[must_use]
    pub fn depends_on(
        &self,
        variables: &[Variable<TYPE, ERR>],
        f: impl 'static + Fn(&[TYPE]) -> Result<TYPE, ERR>,
    ) -> Constraint<TYPE, ERR> {
        Constraint::Computed(ComputedConstraint {
            depends_on: variables.to_vec(),
            compute: Box::new(f),
        })
    }
}

impl<TYPE: Type, ERR> Constraint<TYPE, ERR> {
    pub fn and(self, other: Constraint<TYPE, ERR>) -> Constraint<TYPE, ERR> {
        Constraint::And(Box::new(self), Box::new(other))
    }

    // TODO: if you ever want to build a csp solver, feel free to uncomment these
    // pub fn or(self, other: Constraint<TYPE, ERR>) -> Constraint<TYPE, ERR> {
    //     Constraint::Or(Box::new(self), Box::new(other))
    // }
    //
    // #[allow(clippy::should_implement_trait)]
    // pub fn not(self) -> Constraint<TYPE, ERR> {
    //     Not::not(self)
    // }

    pub fn add_to<M: AstInfo, CTX>(self, s: &mut State<M, CTX, TYPE, ERR>) {
        s.add_constraint(self);
    }
}

impl<TYPE: Type, ERR> BitAnd for Constraint<TYPE, ERR> {
    type Output = Constraint<TYPE, ERR>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

// TODO: if you ever want to build a csp solver, feel free to uncomment these
// impl<TYPE: Type, ERR> BitOr for Constraint<TYPE, ERR> {
//     type Output = Constraint<TYPE, ERR>;
//
//     fn bitor(self, rhs: Self) -> Self::Output {
//         self.or(rhs)
//     }
// }

// impl<TYPE: Type, ERR> Not for Constraint<TYPE, ERR> {
//     type Output = Constraint<TYPE, ERR>;
//
//     fn not(self) -> Self::Output {
//         Constraint::Not(Box::new(self))
//     }
// }
