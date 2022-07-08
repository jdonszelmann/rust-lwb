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
impl<M: AstInfo> AstNode<M> for Meta<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "start"
    }
    fn sort(&self) -> &'static str {
        "meta"
    }
}
impl<M: AstInfo> AstNode<M> for DocComment<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "doc-comment"
    }
    fn sort(&self) -> &'static str {
        "doc-comment"
    }
}
impl<M: AstInfo> AstNode<M> for Newline<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Unix(meta, ..) => meta,
            Self::Windows(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Unix(..) => "unix",
            Self::Windows(..) => "windows",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "newline"
    }
}
impl<M: AstInfo> AstNode<M> for SortOrMeta<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Meta(meta, ..) => meta,
            Self::Sort(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Meta(..) => "meta",
            Self::Sort(..) => "sort",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "sort-or-meta"
    }
}
impl<M: AstInfo> AstNode<M> for Layout<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Simple(meta, ..) => meta,
            Self::Comment(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Simple(..) => "simple",
            Self::Comment(..) => "comment",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "layout"
    }
}
impl<M: AstInfo> AstNode<M> for Number<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "number"
    }
    fn sort(&self) -> &'static str {
        "number"
    }
}
impl<M: AstInfo> AstNode<M> for AnnotationList<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "annotation-list"
    }
    fn sort(&self) -> &'static str {
        "annotation-list"
    }
}
impl<M: AstInfo> AstNode<M> for Constructor<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::ConstructorDocumented(meta, ..) => meta,
            Self::Constructor(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::ConstructorDocumented(..) => "constructor-documented",
            Self::Constructor(..) => "constructor",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "constructor"
    }
}
impl<M: AstInfo> AstNode<M> for Identifier<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "identifier"
    }
    fn sort(&self) -> &'static str {
        "identifier"
    }
}
impl<M: AstInfo> AstNode<M> for CharacterClass<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "class"
    }
    fn sort(&self) -> &'static str {
        "character-class"
    }
}
impl<M: AstInfo> AstNode<M> for Sort<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::SortDocumented(meta, ..) => meta,
            Self::Sort(meta, ..) => meta,
            Self::SortSingle(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::SortDocumented(..) => "sort-documented",
            Self::Sort(..) => "sort",
            Self::SortSingle(..) => "sort-single",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "sort"
    }
}
impl<M: AstInfo> AstNode<M> for StringChar<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Escaped(meta, ..) => meta,
            Self::Normal(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Escaped(..) => "escaped",
            Self::Normal(..) => "normal",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "string-char"
    }
}
impl<M: AstInfo> AstNode<M> for Expression<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Star(meta, ..) => meta,
            Self::Plus(meta, ..) => meta,
            Self::Maybe(meta, ..) => meta,
            Self::RepeatExact(meta, ..) => meta,
            Self::RepeatRange(meta, ..) => meta,
            Self::RepeatLower(meta, ..) => meta,
            Self::Literal(meta, ..) => meta,
            Self::SingleQuoteLiteral(meta, ..) => meta,
            Self::Delimited(meta, ..) => meta,
            Self::Sort(meta, ..) => meta,
            Self::Class(meta, ..) => meta,
            Self::Paren(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Star(..) => "star",
            Self::Plus(..) => "plus",
            Self::Maybe(..) => "maybe",
            Self::RepeatExact(..) => "repeat-exact",
            Self::RepeatRange(..) => "repeat-range",
            Self::RepeatLower(..) => "repeat-lower",
            Self::Literal(..) => "literal",
            Self::SingleQuoteLiteral(..) => "single-quote-literal",
            Self::Delimited(..) => "delimited",
            Self::Sort(..) => "sort",
            Self::Class(..) => "class",
            Self::Paren(..) => "paren",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "expression"
    }
}
impl<M: AstInfo> AstNode<M> for DelimitedBound<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::NumNum(meta, ..) => meta,
            Self::NumInf(meta, ..) => meta,
            Self::Num(meta, ..) => meta,
            Self::Star(meta, ..) => meta,
            Self::Plus(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::NumNum(..) => "num-num",
            Self::NumInf(..) => "num-inf",
            Self::Num(..) => "num",
            Self::Star(..) => "star",
            Self::Plus(..) => "plus",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "delimited-bound"
    }
}
impl<M: AstInfo> AstNode<M> for Program<M> {
    fn ast_info(&self) -> &M {
        let Self(meta, ..) = self;
        meta
    }
    fn constructor(&self) -> &'static str {
        "program"
    }
    fn sort(&self) -> &'static str {
        "program"
    }
}
impl<M: AstInfo> AstNode<M> for Annotation<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Injection(meta, ..) => meta,
            Self::NoPrettyPrint(meta, ..) => meta,
            Self::SingleString(meta, ..) => meta,
            Self::NoLayout(meta, ..) => meta,
            Self::Hidden(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Injection(..) => "injection",
            Self::NoPrettyPrint(..) => "no-pretty-print",
            Self::SingleString(..) => "single-string",
            Self::NoLayout(..) => "no-layout",
            Self::Hidden(..) => "hidden",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "annotation"
    }
}
impl<M: AstInfo> AstNode<M> for CharacterClassItem<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Range(meta, ..) => meta,
            Self::SingleChar(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Range(..) => "range",
            Self::SingleChar(..) => "single-char",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "character-class-item"
    }
}
impl<M: AstInfo> AstNode<M> for EscapeClosingBracket<M> {
    fn ast_info(&self) -> &M {
        match self {
            Self::Escaped(meta, ..) => meta,
            Self::Unescaped(meta, ..) => meta,
            _ => unreachable!(),
        }
    }
    fn constructor(&self) -> &'static str {
        match self {
            Self::Escaped(..) => "escaped",
            Self::Unescaped(..) => "unescaped",
            _ => unreachable!(),
        }
    }
    fn sort(&self) -> &'static str {
        "escape-closing-bracket"
    }
}
