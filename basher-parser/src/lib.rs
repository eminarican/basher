extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::error::Error;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "bash.pest"]
struct BashParser;

pub fn parse_program(input: &str) -> Result<Vec<Expr>, Error<Rule>> {
    let pairs = BashParser::parse(Rule::program, input)?;
    Ok(pairs.map(|pair| Expr::parse(pair)).collect())
}

pub type Scope = Vec<Expr>;

#[derive(Debug)]
pub enum Expr {
    Func(Func),
    Chain(Chain),
}

#[derive(Debug)]
pub struct Func {
    ident: String,
    body: Scope,
}

pub type Chain = Vec<ChainElem>;

#[derive(Debug)]
pub enum ChainElem {
    Call(Call),
    Op(Operator),
}

pub type Call = Vec<String>;

#[derive(Debug)]
pub enum Operator {
    Redir,
    Pipe,
    And,
}

pub trait NodeParser {
    fn parse(pair: Pair<Rule>) -> Self;
}

impl NodeParser for Scope {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut exprs = vec![];

        for pair in pair.into_inner() {
            exprs.push(Expr::parse(pair))
        }

        exprs
    }
}

impl NodeParser for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::func => Self::Func(Func::parse(pair)),
            Rule::chain => Self::Chain(Chain::parse(pair)),
            _ => unreachable!()
        }
    }
}

impl NodeParser for Func {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let mut next = || inner.next().unwrap();

        Self {
            ident: next().as_str().to_string(),
            body: Scope::parse(next()),
        }
    }
}

impl NodeParser for Chain {
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

impl NodeParser for Call {
    fn parse(pair: Pair<Rule>) -> Self {
        pair.into_inner().map(|p| p.as_str().to_string()).collect()
    }
}

impl NodeParser for Operator {
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
