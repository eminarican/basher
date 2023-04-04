extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::error::Error;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "bash.pest"]
struct BashParser;

pub type BashError = Error<Rule>;

pub fn parse(input: &str) -> Result<Scope, BashError> {
    let pairs = BashParser::parse(Rule::program, input)?;
    Ok(pairs.map(|pair| Expr::parse(pair)).collect())
}

pub type Scope = Vec<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
    Func(Func),
    Chain(Chain),
}

#[derive(Debug, Clone)]
pub struct Func {
    pub ident: String,
    pub body: Scope,
}

pub type Chain = Vec<ChainElem>;

#[derive(Debug, Clone)]
pub enum ChainElem {
    Call(Call),
    Op(Operator),
}

pub type Call = Vec<String>;

#[derive(Debug, Clone)]
pub enum Operator {
    Redir,
    Pipe,
    And,
}

pub trait NodeParse {
    fn parse(pair: Pair<Rule>) -> Self;
}

impl NodeParse for Scope {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut exprs = vec![];

        for pair in pair.into_inner() {
            exprs.push(Expr::parse(pair))
        }

        exprs
    }
}

impl NodeParse for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::func => Self::Func(Func::parse(pair)),
            Rule::chain => Self::Chain(Chain::parse(pair)),
            _ => unreachable!()
        }
    }
}

impl NodeParse for Func {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let mut next = || inner.next().unwrap();

        Self {
            ident: next().as_str().to_string(),
            body: Scope::parse(next()),
        }
    }
}

impl NodeParse for Chain {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut elems = vec![];

        for pair in pair.into_inner() {
            let elem = match pair.as_rule() {
                Rule::ops => ChainElem::Op(Operator::parse(pair)),
                Rule::call => ChainElem::Call(Call::parse(pair)),
                _ => unreachable!()
            };

            elems.push(elem);
        }

        elems
    }
}

impl NodeParse for Call {
    fn parse(pair: Pair<Rule>) -> Self {
        pair.into_inner().map(|p| p.as_str().to_string()).collect()
    }
}

impl NodeParse for Operator {
    fn parse(pair: Pair<Rule>) -> Self {
        let item = pair.into_inner().next().unwrap();

        match item.as_rule() {
            Rule::redir => Self::Redir,
            Rule::pipe => Self::Pipe,
            Rule::and => Self::And,
            _ => unreachable!()
        }
    }
}
