use crate::error::display_miette_error;
use crate::sources::span::Span;
use crate::typechecker::constraints::KnownVariable;
use crate::typechecker::Type;
use miette::{Diagnostic, LabeledSpan, Severity, SourceCode};
use std::fmt::{Debug, Display};
use std::rc::Rc;
use thiserror::Error;

pub trait CustomTypeError: Debug + Display {}

#[derive(Error, Debug)]
pub enum GeneratedTypeError<TYPE: Type> {
    #[error("can't unify {:?} and {:?}", _1.value, _3.value)]
    CantUnify(
        Option<Span>,
        Rc<KnownVariable<TYPE>>,
        Option<Span>,
        Rc<KnownVariable<TYPE>>,
    ),
}

impl<TYPE: Type> Diagnostic for GeneratedTypeError<TYPE> {
    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            GeneratedTypeError::CantUnify(Some(a), _, _, _) => Some(&a.source),
            GeneratedTypeError::CantUnify(_, _, Some(a), _) => Some(&a.source),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let mut res = Vec::new();

        match self {
            GeneratedTypeError::CantUnify(a_span, a, b_span, b) => {
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
