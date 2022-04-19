#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]
// |==========================================================|
// |      WARNING: THIS FILE IS AUTOMATICALLY GENERATED.      |
// |      CHANGES TO IT WILL BE DELETED WHEN REGENERATED.     |
// | IN GENERAL, THIS FILE SHOULD NOT BE MODIFIED IN ANY WAY. |
// |==========================================================|
use super::prelude::*;
impl<M: AstInfo> FromPairs<M> for Identifier<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "identifier");
        let info = generator.generate(&pair);
        return Self(info, pair.constructor_value.span().as_str().to_string());
    }
}
impl<M: AstInfo> FromPairs<M> for Int<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "int");
        let info = generator.generate(&pair);
        Self(
            info,
            if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                l . iter () . map (| x | if let ParsePairExpression :: Empty (ref span) = x { span . as_str () . to_string () } else { unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "int") ; }) . collect ()
            } else {
                unreachable!(
                    "expected different parse pair expression in pair to ast conversion of {}",
                    "int"
                );
            },
        )
    }
}
impl<M: AstInfo> FromPairs<M> for Bool<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "bool");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "true" => Self::True(info),
            "false" => Self::False(info),
            a => unreachable!("{}", a),
        }
    }
}
impl<M: AstInfo> FromPairs<M> for Expression<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "expression");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "add" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Add(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                        if let ParsePairExpression::Sort(_, ref s) = l[2usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            "sub" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Sub(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                        if let ParsePairExpression::Sort(_, ref s) = l[2usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            "eq" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Eq(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                        if let ParsePairExpression::Sort(_, ref s) = l[2usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            "index" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Index(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                        if let ParsePairExpression::Sort(_, ref s) = l[2usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            "list" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::List(
                        info,
                        if let ParsePairExpression::List(_, ref l) = l[1usize] {
                            l . iter () . map (| x | if let ParsePairExpression :: Sort (_ , ref s) = x { Box :: new (Expression :: from_pairs (s , generator)) } else { unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression") ; }) . collect ()
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            "bool" => {
                Self::Bool(
                    info,
                    if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                        Box::new(Bool::from_pairs(s, generator))
                    } else {
                        unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                    },
                )
            }
            "int" => {
                Self::Int(
                    info,
                    if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                        Box::new(Int::from_pairs(s, generator))
                    } else {
                        unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                    },
                )
            }
            "identifier" => {
                Self::Identifier(
                    info,
                    if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                        Box::new(Identifier::from_pairs(s, generator))
                    } else {
                        unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                    },
                )
            }
            "paren" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Paren(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[1usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "expression");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "expression"
                    );
                }
            }
            a => unreachable!("{}", a),
        }
    }
}
impl<M: AstInfo> FromPairs<M> for Statement<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "statement");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "if" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::If(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[1usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement");
                        },
                        if let ParsePairExpression::List(_, ref l) = l[3usize] {
                            l . iter () . map (| x | if let ParsePairExpression :: Sort (_ , ref s) = x { Box :: new (Statement :: from_pairs (s , generator)) } else { unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement") ; }) . collect ()
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "statement"
                    );
                }
            }
            "expression" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Expression(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "statement"
                    );
                }
            }
            "assignment" => {
                if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                    Self::Assignment(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = l[0usize] {
                            Box::new(Identifier::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement");
                        },
                        if let ParsePairExpression::Sort(_, ref s) = l[2usize] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "statement");
                        },
                    )
                } else {
                    unreachable!(
                        "expected different parse pair expression in pair to ast conversion of {}",
                        "statement"
                    );
                }
            }
            a => unreachable!("{}", a),
        }
    }
}
impl<M: AstInfo> FromPairs<M> for Program<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "program");
        let info = generator.generate(&pair);
        Self(
            info,
            if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                l . iter () . map (| x | if let ParsePairExpression :: Sort (_ , ref s) = x { Box :: new (Statement :: from_pairs (s , generator)) } else { unreachable ! ("expected different parse pair expression in pair to ast conversion of {}" , "program") ; }) . collect ()
            } else {
                unreachable!(
                    "expected different parse pair expression in pair to ast conversion of {}",
                    "program"
                );
            },
        )
    }
}
impl<M: AstInfo> FromPairs<M> for Layout<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "layout");
        let info = generator.generate(&pair);
        Self(
            info,
            if let ParsePairExpression::Empty(ref span) = pair.constructor_value {
                span.as_str().to_string()
            } else {
                unreachable!(
                    "expected different parse pair expression in pair to ast conversion of {}",
                    "layout"
                );
            },
        )
    }
}
