#![feature(fn_traits)]

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use basher_parser::{BashError, Chain, ChainElem, Expr, Func, Scope, Operator};
use basher_parser::parse;

pub fn eval<F>(input: &str, callback: F) -> Result<Vec<String>, BashError>
    where for<'a> F: FnMut(String, Vec<String>, bool) -> Vec<String> + 'a,
{
    let ctx = NodeEvalCtx::new(Rc::new(RefCell::new(callback)));
    Ok(parse(input)?.eval(ctx))
}

trait NodeEval {
    fn eval(&self, ctx: NodeEvalCtx) -> Vec<String>;
}

#[derive(Clone)]
struct NodeEvalCtx {
    funcs: HashMap<String, Func>,
    callback: Rc<RefCell<dyn FnMut(String, Vec<String>, bool) -> Vec<String>>>,
}

impl NodeEvalCtx {
    fn new(callback: Rc<RefCell<dyn FnMut(String, Vec<String>, bool) -> Vec<String>>>) -> Self {
        Self {
            funcs: HashMap::new(),
            callback,
        }
    }
}

impl NodeEval for Scope {
    fn eval(&self, mut ctx: NodeEvalCtx) -> Vec<String> {
        let mut output = vec![];

        for expr in self {
            if let Expr::Func(func) = expr {
                ctx.funcs.insert(func.ident.clone(), func.clone());
                continue
            }

            output.append(&mut expr.eval(ctx.clone()))
        }

        output
    }
}

impl NodeEval for Expr {
    fn eval(&self, ctx: NodeEvalCtx) -> Vec<String> {
        match self {
            Expr::Func(_) => vec![],
            Expr::Chain(chain) => chain.eval(ctx),
        }
    }
}

impl NodeEval for Func {
    fn eval(&self, ctx: NodeEvalCtx) -> Vec<String> {
        self.body.eval(ctx)
    }
}

impl NodeEval for Chain {
    fn eval(&self, mut ctx: NodeEvalCtx) -> Vec<String> {
        let mut output = vec![];

        let mut operator: Option<Operator> = None;
        let mut last: Option<Vec<String>> = None;

        for elem in self {
            match elem {
                ChainElem::Call(call) => {
                    let mut call = call.clone();
                    let mut piped = false;

                    match operator.as_ref().unwrap_or(&Operator::And) {
                        Operator::Redir => {}
                        Operator::Pipe => {
                            if let Some(mut output) = last.take() {
                                piped = true;
                                call.append(&mut output)
                            }
                        }
                        Operator::And => {
                            if let Some(mut last) = last.take() {
                                output.append(&mut last)
                            }
                        }
                    }

                    let name = call[0].clone();
                    let args = call[1..].to_vec();

                    last = Some(if let Some(func) = ctx.funcs.get(&name) {
                        func.eval(ctx.clone())
                    } else {
                        ctx.callback.borrow_mut().call_mut((name, args, piped))
                    });
                },
                ChainElem::Op(op) => operator = Some(op.clone()),
            }
        }

        if let Some(mut last) = last.take() {
            output.append(&mut last)
        }

        output
    }
}
