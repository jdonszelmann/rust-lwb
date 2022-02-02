#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]
// |==========================================================|
// | WARNING: THIS FILE IS AUTOMATICALLY GENERATED.           |
// | CHANGES TO IT WILL BE DELETED WHEN REGENERATED.          |
// | IN GENERAL, THIS FILE SHOULD NOT BE MODIFIED IN ANY WAY. |
// |==========================================================|

use crate::codegen_prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Identifier<M: AstInfo> {
    Identifier(M, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EscapeClosingBracket<M: AstInfo> {
    Escaped(M, String),
    Unescaped(M, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterClassItem<M: AstInfo> {
    Range(
        M,
        Box<EscapeClosingBracket<M>>,
        Box<EscapeClosingBracket<M>>,
    ),
    SingleChar(M, Box<EscapeClosingBracket<M>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StringChar<M: AstInfo> {
    Escaped(M, String),
    Normal(M, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Number<M: AstInfo> {
    Number(M, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterClass<M: AstInfo> {
    Class(M, bool, Vec<Box<CharacterClassItem<M>>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Expression<M: AstInfo> {
    Star(M, Box<Expression<M>>),
    Plus(M, Box<Expression<M>>),
    Maybe(M, Box<Expression<M>>),
    RepeatExact(
        M,
        Box<Expression<M>>,
        Box<Number<M>>,
        Option<Box<Number<M>>>,
    ),
    Literal(M, Vec<Box<StringChar<M>>>),
    Sort(M, Box<Identifier<M>>),
    Class(M, Box<CharacterClass<M>>),
    Paren(M, Vec<Box<Expression<M>>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Annotation<M: AstInfo> {
    Annotation(M, Option<Box<Identifier<M>>>, Vec<Box<Identifier<M>>>, bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Constructor<M: AstInfo> {
    Constructor(
        M,
        Box<Identifier<M>>,
        Vec<Box<Expression<M>>>,
        Option<Box<Annotation<M>>>,
    ),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Newline<M: AstInfo> {
    Unix(M),
    Windows(M),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Sort<M: AstInfo> {
    Sort(M, Box<Identifier<M>>, Vec<Box<Constructor<M>>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Meta<M: AstInfo> {
    Layout(M, Box<CharacterClass<M>>),
    Start(M, Box<Identifier<M>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SortOrMeta<M: AstInfo> {
    Meta(M, Box<Meta<M>>),
    Sort(M, Box<Sort<M>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Program<M: AstInfo> {
    Program(M, Vec<Box<SortOrMeta<M>>>),
}

pub type AST_ROOT<M> = Program<M>;

impl<M: AstInfo> FromPairs<M> for Identifier<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "identifier");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "identifier" => {
                Self::Identifier(info, pair.constructor_value.span().as_str().to_string())
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Identifier<M> {}

impl<M: AstInfo> FromPairs<M> for EscapeClosingBracket<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "escape-closing-bracket");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "escaped" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Escaped(
                        info,
                        if let ParsePairExpression::Empty(ref span) = p[1] {
                            span.as_str().to_string()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of escape-closing-bracket")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of escape-closing-bracket")
                }
            }
            "unescaped" => Self::Unescaped(
                info,
                if let ParsePairExpression::Empty(ref span) = pair.constructor_value {
                    span.as_str().to_string()
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of escape-closing-bracket")
                },
            ),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for EscapeClosingBracket<M> {}

impl<M: AstInfo> FromPairs<M> for CharacterClassItem<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "character-class-item");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "range" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Range(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(EscapeClosingBracket::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of character-class-item")
                        },
                        if let ParsePairExpression::Sort(_, ref s) = p[2] {
                            Box::new(EscapeClosingBracket::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of character-class-item")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of character-class-item")
                }
            }
            "single-char" => Self::SingleChar(
                info,
                if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                    Box::new(EscapeClosingBracket::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of character-class-item")
                },
            ),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for CharacterClassItem<M> {}

impl<M: AstInfo> FromPairs<M> for StringChar<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "string-char");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "escaped" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Escaped(
                        info,
                        if let ParsePairExpression::Empty(ref span) = p[1] {
                            span.as_str().to_string()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of string-char")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of string-char")
                }
            }
            "normal" => Self::Normal(
                info,
                if let ParsePairExpression::Empty(ref span) = pair.constructor_value {
                    span.as_str().to_string()
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of string-char")
                },
            ),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for StringChar<M> {}

impl<M: AstInfo> FromPairs<M> for Number<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "number");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "number" => Self::Number(info, pair.constructor_value.span().as_str().to_string()),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Number<M> {}

impl<M: AstInfo> FromPairs<M> for CharacterClass<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "character-class");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "class" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Class(
                        info,
                        if let ParsePairExpression::List(_, ref l) = p[1] {
                            l.first().is_some()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of character-class")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[2] {
                            l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(CharacterClassItem::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of character-class")
                } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of character-class")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of character-class")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for CharacterClass<M> {}

impl<M: AstInfo> FromPairs<M> for Expression<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "expression");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "star" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Star(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            "plus" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Plus(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            "maybe" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Maybe(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            "repeat-exact" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::RepeatExact(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(Expression::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                        if let ParsePairExpression::Sort(_, ref s) = p[2] {
                            Box::new(Number::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[4] {
                            l.first().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Number::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                } })
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            "literal" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Literal(
                        info,
                        if let ParsePairExpression::List(_, ref l) = p[1] {
                            l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(StringChar::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            "sort" => Self::Sort(
                info,
                if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                    Box::new(Identifier::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                },
            ),
            "class" => Self::Class(
                info,
                if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                    Box::new(CharacterClass::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                },
            ),
            "paren" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Paren(
                        info,
                        if let ParsePairExpression::List(_, ref l) = p[1] {
                            l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Expression::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of expression")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of expression")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Expression<M> {}

impl<M: AstInfo> FromPairs<M> for Annotation<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "annotation");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "annotation" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Annotation(
                        info,
                        if let ParsePairExpression::List(_, ref l) = p[1] {
                            l.first().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Identifier::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                } })
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[2] {
                            l.iter().map(|x| { if let ParsePairExpression::List(_, ref p) = x {
            if let ParsePairExpression::Sort(_, ref s) = p[1] {
                    Box::new(Identifier::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                }
        } else {
            panic!("expected empty parse pair expression in pair to ast conversion of annotation")
        } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[3] {
                            l.first().is_some()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of annotation")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Annotation<M> {}

impl<M: AstInfo> FromPairs<M> for Constructor<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "constructor");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "constructor" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Constructor(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[1] {
                            Box::new(Identifier::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[3] {
                            l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Expression::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[5] {
                            l.first().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Annotation::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                } })
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of constructor")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Constructor<M> {}

impl<M: AstInfo> FromPairs<M> for Newline<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "newline");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "unix" => Self::Unix(info),
            "windows" => Self::Windows(info),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Newline<M> {}

impl<M: AstInfo> FromPairs<M> for Sort<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "sort");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "sort" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Sort(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[0] {
                            Box::new(Identifier::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of sort")
                        },
                        if let ParsePairExpression::List(_, ref l) = p[2] {
                            l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(Constructor::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of sort")
                } }).collect()
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of sort")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of sort")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Sort<M> {}

impl<M: AstInfo> FromPairs<M> for Meta<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "meta");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "layout" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Layout(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[2] {
                            Box::new(CharacterClass::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of meta")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of meta")
                }
            }
            "start" => {
                if let ParsePairExpression::List(_, ref p) = pair.constructor_value {
                    Self::Start(
                        info,
                        if let ParsePairExpression::Sort(_, ref s) = p[2] {
                            Box::new(Identifier::from_pairs(s, generator))
                        } else {
                            panic!("expected empty parse pair expression in pair to ast conversion of meta")
                        },
                    )
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of meta")
                }
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Meta<M> {}

impl<M: AstInfo> FromPairs<M> for SortOrMeta<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "sort-or-meta");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "meta" => Self::Meta(
                info,
                if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                    Box::new(Meta::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of sort-or-meta")
                },
            ),
            "sort" => Self::Sort(
                info,
                if let ParsePairExpression::Sort(_, ref s) = pair.constructor_value {
                    Box::new(Sort::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of sort-or-meta")
                },
            ),
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for SortOrMeta<M> {}

impl<M: AstInfo> FromPairs<M> for Program<M> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
        assert_eq!(pair.sort, "program");
        let info = generator.generate(&pair);
        match pair.constructor_name {
            "program" => {
                Self::Program(
                    info,
                    if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                        l.iter().map(|x| { if let ParsePairExpression::Sort(_, ref s) = x {
                    Box::new(SortOrMeta::from_pairs(s, generator))
                } else {
                    panic!("expected empty parse pair expression in pair to ast conversion of program")
                } }).collect()
                    } else {
                        panic!("expected empty parse pair expression in pair to ast conversion of program")
                    },
                )
            }
            a => unreachable!("{}", a),
        }
    }
}

impl<M: AstInfo> AstNode<M> for Program<M> {}
