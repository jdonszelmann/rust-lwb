use crate::parser::ast::{NodeId, SpannedAstInfo};
use crate::sources::span::Span;
use crate::typechecker::constraints::Variable::{Free, Known};
use crate::typechecker::error::TypeError;
use crate::typechecker::state::State;
use crate::typechecker::Type;
use itertools::Itertools;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{BitAnd, DerefMut};
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
pub struct KnownVariable<TYPE: Type> {
    dbg_msg: String,
    id: VariableId,
    pub(crate) value: TYPE,
    pub(crate) span: RefCell<Option<Span>>,
}

#[derive(Debug)]
pub struct FreeVariable {
    dbg_msg: String,
    id: VariableId,
    pub(crate) span: RefCell<Option<Span>>,
}

pub enum Variable<TYPE: Type> {
    Free(Rc<FreeVariable>),
    Known(Rc<KnownVariable<TYPE>>),
}

impl<TYPE: Type> Debug for Variable<TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Free(fv) => write!(
                f,
                "var({}{})",
                fv.id,
                if self.span().is_some() { " + span" } else { "" }
            ),
            Known(k) => write!(
                f,
                "var({:?}, {}{})",
                k.value,
                k.id,
                if self.span().is_some() { " + span" } else { "" }
            ),
        }
    }
}

impl<TYPE: Type> Variable<TYPE> {
    pub fn is_known(&self) -> bool {
        match self {
            Free(_) => false,
            Known(_) => true,
        }
    }

    pub(crate) fn new_free(span: Option<Span>, dbg_msg: impl AsRef<str>) -> Self {
        Self::Free(Rc::new(FreeVariable {
            dbg_msg: dbg_msg.as_ref().to_string(),
            id: new_variable_id(),
            span: RefCell::new(span),
        }))
    }

    pub(crate) fn new_known(value: TYPE, span: Option<Span>, dbg_msg: impl AsRef<str>) -> Self {
        Self::Known(Rc::new(KnownVariable {
            dbg_msg: dbg_msg.as_ref().to_string(),
            id: new_variable_id(),
            value,
            span: RefCell::new(span),
        }))
    }

    pub(crate) fn id(&self) -> VariableId {
        match self {
            Free(i) => i.id,
            Known(i) => i.id,
        }
    }

    pub(crate) fn span(&self) -> Ref<Option<Span>> {
        match self {
            Free(i) => i.span.borrow(),
            Known(i) => i.span.borrow(),
        }
    }

    pub(crate) fn dbg_msg(&self) -> &str {
        match self {
            Free(i) => &i.dbg_msg,
            Known(i) => &i.dbg_msg,
        }
    }

    pub(crate) fn span_mut(&self) -> RefMut<Option<Span>> {
        match self {
            Free(i) => i.span.borrow_mut(),
            Known(i) => i.span.borrow_mut(),
        }
    }

    pub(crate) fn merge_span(&self, other: &Variable<TYPE>) {
        let mut sa = self.span_mut();
        let mut sb = other.span_mut();
        match (sa.deref_mut(), sb.deref_mut()) {
            (None, None) => {}
            (a @ Some(_), b @ None) => *b = a.clone(),
            (a @ None, b @ Some(_)) => *a = b.clone(),
            (Some(_a), Some(_b)) => {
                // both have a span already so nothing needs to be done

                // this is bad and doesn't give nice errors:
                // let new_span = a.merge(b);
                // *a = new_span.clone();
                // *b = new_span;
            }
        }
    }
}

impl<TYPE: Type> Clone for Variable<TYPE> {
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
    impl<TYPE: Type> Sealed for Variable<TYPE> {}
    impl<TYPE: Type> Sealed for &Variable<TYPE> {}
    impl<TYPE: Type> Sealed for TYPE {}
}

/// Converts things into variables. Trivially, variables
/// are convertible into variables. But types also are.
/// This trait can be seen in [`Variable::equiv`]
pub trait IntoVariable<TYPE: Type>: sealed::Sealed {
    fn into(self) -> Variable<TYPE>;
}

impl<TYPE: Type> IntoVariable<TYPE> for TYPE {
    fn into(self) -> Variable<TYPE> {
        Variable::new_known(self, None, "")
    }
}

impl<TYPE: Type> IntoVariable<TYPE> for Variable<TYPE> {
    fn into(self) -> Variable<TYPE> {
        self
    }
}

impl<TYPE: Type> IntoVariable<TYPE> for &Variable<TYPE> {
    fn into(self) -> Variable<TYPE> {
        self.clone()
    }
}

pub type ComputerConstraintResult<TYPE> = Result<TYPE, TypeError<TYPE>>;
pub struct ComputedConstraint<TYPE: Type> {
    depends_on: Vec<Variable<TYPE>>,
    #[allow(dead_code)]
    compute: Box<dyn Fn(&[TYPE]) -> ComputerConstraintResult<TYPE>>,
}

pub enum Constraint<TYPE: Type> {
    And(Box<Self>, Box<Self>),

    // TODO: if you ever want to build a csp solver, feel free to uncomment these
    // Or(Box<Self>, Box<Self>),
    // Not(Box<Self>),
    Equiv(Variable<TYPE>, Variable<TYPE>),
    NotEquiv(Variable<TYPE>, Variable<TYPE>),
    Node(Variable<TYPE>, NodeId),

    Computed(ComputedConstraint<TYPE>),
}

impl<TYPE: Type> Debug for Constraint<TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::And(a, b) => write!(f, "{:?} & {:?}", a, b),
            Constraint::Equiv(a, b) => write!(f, "{:?} == {:?}", a, b),
            Constraint::NotEquiv(a, b) => write!(f, "{:?} != {:?}", a, b),
            Constraint::Node(var, nodeid) => {
                write!(f, "{:?} == {:?}", nodeid, var)
            }
            Constraint::Computed(ComputedConstraint { depends_on, .. }) => {
                write!(
                    f,
                    "computed constraint depending on [{}]",
                    depends_on.iter().map(|i| format!("{:?}", i)).join(",")
                )
            }
        }
    }
}

impl<TYPE: Type> Constraint<TYPE> {
    pub fn variables(&self) -> Vec<&Variable<TYPE>> {
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
    #[must_use]
    pub fn equiv(&self, other: impl IntoVariable<TYPE>) -> Constraint<TYPE> {
        Constraint::Equiv(self.clone(), other.into())
    }

    #[must_use]
    pub fn not_equiv(&self, other: impl IntoVariable<TYPE>) -> Constraint<TYPE> {
        Constraint::NotEquiv(self.clone(), other.into())
    }

    #[must_use]
    pub fn all_equiv<const N: usize>(&self, variables: [Variable<TYPE>; N]) -> Constraint<TYPE> {
        assert!(variables.len() >= 2);

        let mut res: Option<Constraint<TYPE>> = None;

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
        variables: &[Variable<TYPE>],
        f: impl 'static + Fn(&[TYPE]) -> ComputerConstraintResult<TYPE>,
    ) -> Constraint<TYPE> {
        Constraint::Computed(ComputedConstraint {
            depends_on: variables.to_vec(),
            compute: Box::new(f),
        })
    }
}

impl<TYPE: Type> Constraint<TYPE> {
    pub fn and(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
        Constraint::And(Box::new(self), Box::new(other))
    }

    // TODO: if you ever want to build a csp solver, feel free to uncomment these
    // pub fn or(self, other: Constraint<TYPE>) -> Constraint<TYPE> {
    //     Constraint::Or(Box::new(self), Box::new(other))
    // }
    //
    // #[allow(clippy::should_implement_trait)]
    // pub fn not(self) -> Constraint<TYPE> {
    //     Not::not(self)
    // }

    pub fn add_to<M: SpannedAstInfo, CTX>(self, s: &mut State<M, CTX, TYPE>) {
        s.add_constraint(self);
    }
}

impl<TYPE: Type> BitAnd for Constraint<TYPE> {
    type Output = Constraint<TYPE>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

// TODO: if you ever want to build a csp solver, feel free to uncomment these
// impl<TYPE: Type> BitOr for Constraint<TYPE> {
//     type Output = Constraint<TYPE>;
//
//     fn bitor(self, rhs: Self) -> Self::Output {
//         self.or(rhs)
//     }
// }

// impl<TYPE: Type> Not for Constraint<TYPE> {
//     type Output = Constraint<TYPE>;
//
//     fn not(self) -> Self::Output {
//         Constraint::Not(Box::new(self))
//     }
// }
