use std::error::Error;
use crate::error::display_miette_error;
use crate::sources::span::Span;
use crate::typechecker::constraints::KnownVariable;
use crate::typechecker::Type;
use miette::{Diagnostic, LabeledSpan, Severity, SourceCode};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use thiserror::Error;

pub trait CustomTypeError: Debug + Display {}

#[derive(Debug)]
pub enum GeneratedTypeError<TYPE: Type> {
    CantUnify(
        Option<Span>,
        Rc<KnownVariable<TYPE>>,
        bool,
        Option<Span>,
        Rc<KnownVariable<TYPE>>,
        bool,
    ),
}

impl<TYPE: Type> Error for GeneratedTypeError<TYPE> {}

impl<TYPE: Type> Display for GeneratedTypeError<TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratedTypeError::CantUnify(_, at, true, _, bt, true) => {
                write!(f, "can't unify {:?} with {:?}", at.value, bt.value)
            }
            GeneratedTypeError::CantUnify(_, at, true, _, bt, false) |
            GeneratedTypeError::CantUnify(_, bt, false, _, at, true) => {
                write!(f, "can't use value with type {:?} in a place where {:?} is expected", bt.value, at.value)
            }
            GeneratedTypeError::CantUnify(_, bt, false, _, at, false) => {
                write!(f, "mistake in the type specification of the language: both {:?} and {:?} are expected at the same time", bt.value, at.value)
            }
        }
    }
}

impl<TYPE: Type> Diagnostic for GeneratedTypeError<TYPE> {
    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            GeneratedTypeError::CantUnify(Some(a), _, _, _, _, _) => Some(&a.source),
            GeneratedTypeError::CantUnify(_, _, _, Some(a), _, _) => Some(&a.source),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let mut res = Vec::new();

        match self {
            GeneratedTypeError::CantUnify(a_span, a, _, b_span, b, _) => {
                if let Some(i) = a_span {
                    res.push(LabeledSpan::new_with_span(
                        Some(format!("{:?}", a.value)),
                        i.clone(),
                    ));
                }
                if let Some(i) = b_span {
                    res.push(LabeledSpan::new_with_span(
                        Some(format!("{:?}", b.value)),
                        i.clone(),
                    ));
                }
            }
        }

        Some(Box::new(res.into_iter()))
    }
}

#[derive(Error, Debug)]
pub enum TypeError<TYPE: Type> {
    #[error("{}", display_miette_error(_0))]
    Generated(GeneratedTypeError<TYPE>),
    #[error("custom error")]
    Custom(Box<dyn CustomTypeError>),
}

impl<T: 'static + CustomTypeError, TYPE: Type> From<T> for TypeError<TYPE> {
    fn from(t: T) -> Self {
        Self::Custom(Box::new(t))
    }
}

impl<TYPE: Type> From<GeneratedTypeError<TYPE>> for TypeError<TYPE> {
    fn from(g: GeneratedTypeError<TYPE>) -> Self {
        Self::Generated(g)
    }
}
