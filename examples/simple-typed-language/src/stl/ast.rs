#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]
// |==========================================================|
// |      WARNING: THIS FILE IS AUTOMATICALLY GENERATED.      |
// |      CHANGES TO IT WILL BE DELETED WHEN REGENERATED.     |
// | IN GENERAL, THIS FILE SHOULD NOT BE MODIFIED IN ANY WAY. |
// |==========================================================|
// Generated at 17/04/2022 23:43:35 +02:00 - 17/04/2022 21:43:35 UTC
use super::prelude::*;

#[derive(Debug)]
pub struct Identifier<M : AstInfo>(pub M, pub String);

#[derive(Debug)]
pub struct Int<M : AstInfo>(pub M, pub Vec<String>);

#[derive(Debug)]
pub enum Bool<M : AstInfo> {
    True(M, ),
    False(M, ),
}

#[derive(Debug)]
pub enum Expression<M : AstInfo> {
    Add(M, Box<Expression<M>>, Box<Expression<M>>, ),
    Sub(M, Box<Expression<M>>, Box<Expression<M>>, ),
    Eq(M, Box<Expression<M>>, Box<Expression<M>>, ),
    Index(M, Box<Expression<M>>, Box<Expression<M>>, ),
    List(M, Vec<Box<Expression<M>>>),
    Bool(M, Box<Bool<M>>),
    Int(M, Box<Int<M>>),
    Identifier(M, Box<Identifier<M>>),
    Paren(M, Box<Expression<M>>),
}

#[derive(Debug)]
pub enum Statement<M : AstInfo> {
    If(M, Box<Expression<M>>, Vec<Box<Statement<M>>>, ),
    Expression(M, Box<Expression<M>>),
    Assignment(M, Box<Identifier<M>>, Box<Expression<M>>, ),
}

#[derive(Debug)]
pub struct Program<M : AstInfo>(pub M, pub Vec<Box<Statement<M>>>);

#[derive(Debug)]
pub struct Layout<M : AstInfo>(pub M, pub String);

pub type AST_ROOT<M> = Program<M>;