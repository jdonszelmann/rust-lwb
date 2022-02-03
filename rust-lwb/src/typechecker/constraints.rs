use crate::parser::ast::NodeId;
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::Type;
use std::ops::{BitAnd, BitOr, Not};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBALLY_UNIQUE_VARIABLE_ID: AtomicU64 = AtomicU64::new(0);
pub fn new_variable_id() -> VariableId {
    let value = GLOBALLY_UNIQUE_VARIABLE_ID.fetch_add(1, Ordering::Relaxed);
    VariableId(value)
}

#[derive(Copy, Clone)]
pub struct VariableId(pub(super) u64);

pub struct KnownVariable<TYPE> {
    pub(super) id: VariableId,
    pub(super) value: TYPE,
}

pub struct FreeVariable {
    pub(super) id: VariableId,
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
    Free(Rc<FreeVariable>),
    Known(Rc<KnownVariable<TYPE>>),
}

impl<TYPE> Clone for Variable<TYPE> {
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

    pub trait Sealed<TYPE> {}
    impl<TYPE: Type> Sealed<TYPE> for Variable<TYPE> {}
    impl<TYPE: Type> Sealed<TYPE> for TYPE {}
}

/// Converts things into variables. Trivially, variables
/// are convertible into variables. But types also are.
/// This trait can be seen in [`Variable::equiv`]
pub trait IntoVariable<TYPE>: sealed::Sealed<TYPE> {
    fn into(self) -> Variable<TYPE>;
}

impl<TYPE: Type> IntoVariable<TYPE> for TYPE {
    fn into(self) -> Variable<TYPE> {
        Variable::Known(Rc::new(KnownVariable {
            id: new_variable_id(),
            value: self,
        }))
    }
}

impl<TYPE: Type> IntoVariable<TYPE> for Variable<TYPE> {
    fn into(self) -> Variable<TYPE> {
        self
    }
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

    None,
}

impl<TYPE: Type> Variable<TYPE> {
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
    pub fn equiv(&self, other: impl IntoVariable<TYPE>) -> Constraint<TYPE> {
        Constraint::Eq(self.clone(), other.into())
    }
}

impl<TYPE: Type> Constraint<TYPE> {
    pub fn and(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
        Constraint::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
        Constraint::Or(Box::new(self), Box::new(other))
    }

    #[allow(clippy::should_implement_trait)]
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
