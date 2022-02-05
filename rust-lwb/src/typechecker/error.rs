use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneratedTypeError<TYPE> {
    #[error("can't unify {0:?} and {1:?}")]
    CantUnify(/*Span,*/ TYPE, TYPE),
}

#[derive(Error, Debug)]
pub enum TypeError<TYPE, ERR> {
    #[error(transparent)]
    Generated(GeneratedTypeError<TYPE>),
    #[error("custom error")]
    Custom(ERR),
}
