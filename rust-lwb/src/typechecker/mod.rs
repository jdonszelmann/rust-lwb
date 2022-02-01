use crate::codegen_prelude::AstInfo;
use crate::parser::ast::{AstNode, NodeId};

struct VariableId(usize);

struct KnownVariable<TYPE> {
    id: VariableId,
    value: TYPE,
}

enum Variable<TYPE> {
    Free(VariableId),
    Known(KnownVariable<TYPE>),
}

trait ComputedConstraint<TYPE: Type> {
    fn solve(&self, input: &Vec<KnownVariable<TYPE>>) -> Constraint<TYPE>;
}

impl<TYPE: Type, F> ComputedConstraint<TYPE> for F where F: Fn(&Vec<TYPE>) -> Constraint<TYPE>  {
    fn solve(&self, input: &Vec<KnownVariable<TYPE>>) -> Constraint<TYPE> {
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
}

trait Type {}

trait TypeCheckable<M: AstInfo, CTX>: AstNode<M> {
    type Type: Type;

    fn generate_constraints(&self) -> Constraint<Self::Type>;
}



// pub enum Expression<M: AstInfo> {
//     Paren(M, Box<Expression<M>>),
//     Literal(M, Option<()>, Vec<Box<StringChar<M>>>),
//     Sort(M, Box<Identifier<M>>),
//     Class(M, Box<CharacterClassItem<M>>),
//     Star(M, Box<Expression<M>>),
//     Plus(M, Box<Expression<M>>),
//     Maybe(M, Box<Expression<M>>),
//     RepeatExact(
//         M,
//         Box<Expression<M>>,
//         Box<Number<M>>,
//         Option<Box<Number<M>>>,
//     ),
//     Sequence(M, Box<Expression<M>>, Box<Expression<M>>),
// }
//
// impl<M: AstInfo> FromPairs<M> for Expression<M> {
//     fn from_pairs<G: GenerateAstInfo<Result=M>>(pair: &ParsePairSort, generator: &mut G) -> Self where Self: Sized {
//         todo!()
//     }
// }
//
// impl<M: AstInfo> AstNode<M> for Expression<M> {}
//
// enum TypeEnum {
//     Int,
// }
//
// impl Type for TypeEnum {}
//
// impl<M: AstInfo, CTX> TypeCheckable<M, CTX> for Expression<M> {
//     type Type = TypeEnum;
//
//     fn generate_constraints(&self, cb: ConstraintBuilder, ctx: &CTX) -> Constraint<Self::Type> {
//         match self {
//             Expression::Paren(_, i) => {
//                 let v1 = cb.free_variable();
//                 let v2 = cb.free_variable();
//                 let v3 = cb.type_of_self(self);
//                 v1.eq(v2);
//
//                 v3.eq(v1);
//
//                 todo!()
//             }
//             Expression::Literal(_, _, _) => {}
//             Expression::Sort(_, _) => {}
//             Expression::Class(_, _) => {}
//             Expression::Star(_, _) => {}
//             Expression::Plus(_, _) => {}
//             Expression::Maybe(_, _) => {}
//             Expression::RepeatExact(_, _, _, _) => {}
//             Expression::Sequence(_, _, _) => {}
//         }
//     }
// }

