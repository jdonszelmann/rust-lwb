use crate::stl::{Program, Statement};
use crate::types::StlType::{EmptyList, List};
use crate::AST::Expression;
use rust_lwb::parser::ast::SpannedAstInfo;
use rust_lwb::typechecker::error::{CustomTypeError, TypeError};
use rust_lwb::typechecker::state::State;
use rust_lwb::typechecker::{Type, TypeCheckable};
use thiserror::Error;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum StlType {
    Int,
    Bool,
    List(Box<StlType>),
    EmptyList,
}

impl Type for StlType {}

#[derive(Error, Debug)]
pub enum StlTypeError {
    #[error("could not unify {0:?} and {0:?} for use in list")]
    NoLub(StlType, StlType),
}

impl CustomTypeError for StlTypeError {}

fn lub(a: &StlType, b: &StlType) -> Result<StlType, TypeError<StlType>> {
    match (a, b) {
        (a, b) if a == b => Ok(a.clone()),

        (EmptyList, a @ List(_)) => Ok(a.clone()),
        (a @ List(_), EmptyList) => Ok(a.clone()),

        (a, b) => Err(StlTypeError::NoLub(a.clone(), b.clone()).into()),
    }
}

fn lub_list(types: &[StlType]) -> Result<StlType, TypeError<StlType>> {
    let mut res = types
        .first()
        .expect("at least one type in lub list")
        .clone();

    for i in &types[1..] {
        res = lub(&res, i)?
    }

    Ok(res)
}

impl<M: SpannedAstInfo> TypeCheckable<M, (), StlType> for Program<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        for i in &self.1 {
            s.type_ok(i);
        }
    }
}

impl<M: SpannedAstInfo> TypeCheckable<M, (), StlType> for Statement<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        match self {
            Statement::If(_, e, block, ..) => {
                let te = s.get_type(e);
                s.add_constraint(te.equiv(StlType::Bool));

                for i in block {
                    s.type_ok(i);
                }
            }
            Statement::Expression(_, e, ..) => s.type_ok(e),
            Statement::Assignment(_, _, e, ..) => {
                // something with scopes: TODO
                s.type_ok(e)
            }
            _ => unreachable!(),
        }
    }
}

impl<M: SpannedAstInfo> TypeCheckable<M, (), StlType> for Expression<M> {
    fn create_constraints<'ast>(&'ast self, s: &mut State<'ast, M, (), StlType>, _: &()) {
        match self {
            Expression::Add(_, a, b, ..) => {
                let ta = s.get_type(a);
                let tb = s.get_type(b);

                s.add_constraint(ta.equiv(tb));
                s.type_of_self(self).equiv(ta).add_to(s);
            }
            Expression::Sub(_, a, b, ..) => {
                let ta = s.get_type(a);
                let tb = s.get_type(b);

                s.add_constraint(ta.equiv(tb));
                s.type_of_self(self).equiv(ta).add_to(s);
            }
            Expression::Int(_, _a, ..) => {
                s.type_of_self(self).equiv(StlType::Int).add_to(s);
            }
            Expression::Identifier(_, _s, ..) => {
                // something with scopes: TODO
            }
            Expression::Eq(_, a, b, ..) => {
                let ta = s.get_type(a);
                let tb = s.get_type(b);

                s.add_constraint(ta.equiv(tb));
                s.type_of_self(self).equiv(StlType::Bool).add_to(s);
            }
            Expression::Index(_, a, b, ..) => s
                .get_type(a)
                .depends_on(&[s.get_type(b)], |types| {
                    Ok(StlType::List(Box::new(types[0].clone())))
                })
                .add_to(s),
            Expression::List(_, exprs, ..) => {
                if !exprs.is_empty() {
                    let tvs: Vec<_> = exprs.iter().map(|i| s.get_type(i)).collect();
                    s.type_of_self(self)
                        .depends_on(&tvs, |types| Ok(StlType::List(Box::new(lub_list(types)?))))
                        .add_to(s);
                } else {
                    s.type_of_self(self).equiv(StlType::EmptyList).add_to(s);
                }
            }
            Expression::Bool(..) => {
                s.type_of_self(self).equiv(StlType::Bool).add_to(s);
            }
            Expression::Paren(_, e, ..) => {
                let tp = s.get_type(&*e);
                s.type_of_self(self).equiv(tp).add_to(s);
            }
            Expression::Testexpr(..) => {}
            _ => unreachable!(),
        }
    }
}
