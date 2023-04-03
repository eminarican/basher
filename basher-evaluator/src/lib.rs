use std::collections::HashMap;

use basher_parser::{BashError, Chain, ChainElem, Expr, Func, Scope, parse, Operator, Call};

pub struct Evaluator<F> {
    scope: Scope,
    funcs: HashMap<String, Func>,
    callback: F,
}

impl<F> Evaluator<F>
    where
        F: FnMut(String, Vec<String>) -> String,
{
    pub fn new(input: &str, callback: F) -> Result<Self, BashError> {
        Ok(Self {
            scope: parse(input)?,
            funcs: HashMap::new(),
            callback,
        })
    }

    pub fn eval(&mut self) {
        self.eval_scope(self.scope.clone(), None);
    }

    fn eval_scope(&mut self, scope: Scope, last_output: Option<String>) -> Option<String> {
        let mut output = last_output;
        for expr in scope.iter() {
            match expr {
                Expr::Func(func) => {
                    self.funcs.insert(func.ident.clone(), func.clone());
                }
                Expr::Chain(chain) => {
                    output = self.eval_chain(chain.clone(), output);
                }
            }
        }
        output
    }

    fn eval_chain(&mut self, chain: Chain, last_output: Option<String>) -> Option<String> {
        let mut output = last_output;
        let mut last_operator = None;

        for elem in chain.iter() {
            match elem {
                ChainElem::Call(call) => {
                    output = self.eval_call(
                        call,
                        output,
                        &mut last_operator,
                    );
                }
                ChainElem::Op(op) => {
                    last_operator = Some(op.clone());
                }
            }
        }
        output
    }

    fn eval_call(&mut self, call: &Call, last_output: Option<String>, last_operator: &mut Option<Operator>, ) -> Option<String> {
        let name = &call[0];

        if let Some(func) = self.funcs.get(name) {
            return self.eval_scope(func.body.clone(), last_output);
        }

        let mut args: Vec<_> = call.iter().skip(1).map(ToString::to_string).collect();

        if let Some(operator) = last_operator {
            match operator {
                Operator::Redir => {}
                Operator::Pipe => {
                    if let Some(output) = &last_output {
                        args.push(output.clone())
                    }
                }
                Operator::And => {}
            }
        }

        Some((self.callback)(name.clone(), args))
    }
}
