use crate::types::Expr;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum EvalResult {
    Err(String),
    Expr(Rc<Expr>),
    Unit,
}

#[derive(Debug)]
pub struct Environment {
    pub contexts: Vec<HashMap<String, (Vec<String>, Rc<Expr>)>>,
}

impl Environment {
    /// Create an empty environment
    pub fn empty() -> Environment {
        Environment {
            contexts: Vec::new(),
        }
    }

    /// Create an environment from a list of variables
    pub fn from_vars(vars: &[(&str, Rc<Expr>)]) -> Environment {
        let mut env = Environment::empty();
        env.push_context();
        vars.iter().for_each(|(name, expr)| {
            let _ = env.add_var(name, expr.clone());
        });
        env
    }

    /// Create a default environment
    pub fn default() -> Environment {
        let mut env = Environment::empty();
        env.push_context();
        let _ = env.add_var("False", Expr::list(&[]));
        let _ = env.add_var("True", Expr::list(&[Expr::fnum(1.0)]));
        env
    }

    /// Look up the given symbol in the environment
    pub fn lookup(&self, symbol: &str) -> Option<(Vec<String>, Rc<Expr>)> {
        self.contexts
            .iter()
            .rev()
            .find(|context| context.contains_key(symbol))
            .map(|context| context.get(symbol))
            .flatten()
            .cloned()
    }

    /// Check whether the given symbol exists in the environment
    pub fn contains_key(&self, symbol: &str) -> bool {
        self.contexts
            .iter()
            .rev()
            .find(|context| context.contains_key(symbol))
            .is_some()
    }

    /// Push a new context on the stack
    pub fn push_context(&mut self) {
        self.contexts.push(HashMap::new());
    }

    /// Pop the last context from the stack
    pub fn pop_context(&mut self) {
        self.contexts.pop();
    }

    /// Add a variable definition to the environment
    pub fn add_var(&mut self, var: &str, val: Rc<Expr>) -> Result<(), String> {
        self.contexts.last_mut().map_or_else(
            || Err("Enviroment has no context!".into()),
            |context| {
                context.insert(var.to_string(), (Vec::new(), val.clone()));
                Ok(())
            },
        )
    }

    /// Add a function definition to the environment
    pub fn add_fn(&mut self, name: &str, params: &[String], body: Rc<Expr>) -> Result<(), String> {
        self.contexts.last_mut().map_or_else(
            || Err("Enviroment has no context!".into()),
            |context| {
                let params = params.iter().map(|s| s.to_string()).collect();
                context.insert(name.to_string(), (params, body));
                Ok(())
            },
        )
    }

    /// Get the number of all contexts
    pub fn num_contexts(&self) -> usize {
        self.contexts.len()
    }
}

/// Evaluate the given expression
pub fn eval(expr: Rc<Expr>, env: &mut Environment) -> EvalResult {
    match &*expr {
        Expr::Symbol(s) => evaluate_symbol(expr.clone(), s, &[], env),
        Expr::FNum(_) => EvalResult::Expr(expr.clone()),
        Expr::List(vals) => {
            if vals.is_empty() {
                return EvalResult::Expr(Expr::list(&[]));
            }

            let op = &*vals[0];
            match op {
                Expr::Symbol(s) if s == "+" => do_math(&vals[1..], env, "+"),
                Expr::Symbol(s) if s == "-" => do_math(&vals[1..], env, "-"),
                Expr::Symbol(s) if s == "*" => do_math(&vals[1..], env, "*"),
                Expr::Symbol(s) if s == "/" => do_math(&vals[1..], env, "/"),

                Expr::Symbol(s) if s == "or" => do_boolean(&vals[1..], env, "or"),
                Expr::Symbol(s) if s == "and" => do_boolean(&vals[1..], env, "and"),
                Expr::Symbol(s) if s == "not" => do_boolean(&vals[1..], env, "not"),

                Expr::Symbol(s) if s == "=" => do_equality(&vals[1..], env, "="),
                Expr::Symbol(s) if s == "!=" => do_equality(&vals[1..], env, "!="),
                Expr::Symbol(s) if s == "if" => if_statement(&vals[1..], env),

                Expr::Symbol(s) if s == "let" => add_var_to_env(&vals[1..], env),
                Expr::Symbol(s) if s == "fn" => add_fun_to_env(&vals[1..], env),
                Expr::Symbol(s) if s == "print" => print(&vals[1..], env),

                Expr::Symbol(s) if env.contains_key(&s) => {
                    evaluate_symbol(expr.clone(), s, &vals[1..], env)
                }
                _ => {
                    let result = vals
                        .iter()
                        .map(|e| eval(e.clone(), env))
                        .filter(|e| *e != EvalResult::Unit)
                        .map(|e| {
                            if let EvalResult::Expr(expr) = e {
                                Ok(expr)
                            } else {
                                Err(e)
                            }
                        })
                        .collect::<Result<Vec<Rc<Expr>>, EvalResult>>();
                    result.map_or_else(|error| error, |expr| EvalResult::Expr(Expr::list(&expr)))
                }
            }
        }
    }
}

/// Evaluate a symbol
fn evaluate_symbol(
    expr: Rc<Expr>,
    symbol: &str,
    args: &[Rc<Expr>],
    env: &mut Environment,
) -> EvalResult {
    env.lookup(symbol).map_or_else(
        || EvalResult::Expr(expr),
        |(param_names, expression)| {
            if param_names.is_empty() {
                eval(expression.clone(), env)
            } else {
                if param_names.len() != args.len() {
                    return EvalResult::Err(format!(
                        "Provided {} arguments but expected {}!",
                        param_names.len(),
                        args.len()
                    ));
                }

                let mapped_args = args
                    .iter()
                    .zip(param_names)
                    .map(|(e, name)| match eval(e.clone(), env) {
                        EvalResult::Err(error) => Err(error),
                        EvalResult::Expr(expr) => Ok((name.to_string(), expr.clone())),
                        EvalResult::Unit => {
                            Err("Cannot pass Unit as an argument to a function!".into())
                        }
                    })
                    .collect::<Result<Vec<(String, Rc<Expr>)>, String>>();

                env.push_context();

                let result = mapped_args.map_or_else(
                    |error| EvalResult::Err(error),
                    |arg_tuples| {
                        arg_tuples.iter().for_each(|(name, expr)| {
                            let _ = env.add_var(name, expr.clone());
                        });
                        eval(expression.clone(), env)
                    },
                );

                env.pop_context();

                result
            }
        },
    )
}

/// Do mathematical operations
/// (+ 1 2 3)
/// (- 1 2 3)
/// (* 1 2 3)
/// (/ 1 2 3)
fn do_math(vals: &[Rc<Expr>], env: &mut Environment, op: &str) -> EvalResult {
    if vals.is_empty() {
        return EvalResult::Err(
            "Mathematical operations must be performed on at least one number!".into(),
        );
    }

    let total = vals
        .iter()
        .map(|e| match eval(e.clone(), env) {
            EvalResult::Err(error) => Err(error),
            EvalResult::Expr(expr) => match &*expr {
                Expr::FNum(n) => Ok(*n),
                _ => Err("Mathematical operations must be performed on numbers!".into()),
            },
            EvalResult::Unit => Err("Mathematical operations must be performed on numbers!".into()),
        })
        .collect::<Result<Vec<f64>, String>>();

    total.map_or_else(
        |error| EvalResult::Err(error),
        |xs| {
            let mut result = xs[0];
            for x in xs.iter().skip(1) {
                match op {
                    "+" => result += x,
                    "-" => result -= x,
                    "*" => result *= x,
                    "/" => result /= x,
                    _ => return EvalResult::Err("Illegal mathematical operation!".into()),
                }
            }
            EvalResult::Expr(Expr::fnum(result))
        },
    )
}

/// Do boolean operations
/// (or True True False)
/// (and True True False)
/// (not True)
fn do_boolean(vals: &[Rc<Expr>], env: &mut Environment, op: &str) -> EvalResult {
    if vals.is_empty() {
        return EvalResult::Err("Boolean operations must be performed on at least value!".into());
    }

    if op == "not" && vals.len() != 1 {
        return EvalResult::Err("Negation must be performed on one symbol!".into());
    }

    let total = vals
        .iter()
        .map(|e| match eval(e.clone(), env) {
            EvalResult::Err(error) => Err(error),
            EvalResult::Expr(expr) => match &*expr {
                Expr::Symbol(s) => match &s[..] {
                    "False" => Ok(false),
                    _ => Ok(true),
                },
                Expr::List(xs) => match xs[..] {
                    [] => Ok(false),
                    _ => Ok(true),
                },
                _ => Err("Boolean operations must be performed symbols!".into()),
            },
            EvalResult::Unit => Err("Boolean operations must be performed on symbols!".into()),
        })
        .collect::<Result<Vec<bool>, String>>();

    total.map_or_else(
        |error| EvalResult::Err(error),
        |xs| {
            let mut result = xs[0];
            for x in xs.iter().skip(1) {
                match op {
                    "or" => result |= x,
                    "and" => result &= x,
                    _ => return EvalResult::Err("Illegal boolean operation!".into()),
                }
            }
            if op == "not" {
                match result {
                    true => EvalResult::Expr(Expr::symbol("False".into())),
                    false => EvalResult::Expr(Expr::symbol("True".into())),
                }
            } else {
                match result {
                    true => EvalResult::Expr(Expr::symbol("True".into())),
                    false => EvalResult::Expr(Expr::symbol("False".into())),
                }
            }
        },
    )
}

/// Do equality operations
/// (= 1 1 1)
/// (!= 1 1 2)
fn do_equality(vals: &[Rc<Expr>], env: &mut Environment, op: &str) -> EvalResult {
    if vals.is_empty() {
        return EvalResult::Err(
            "Equality operations must be performed on at least one value!".into(),
        );
    }

    let total = vals
        .iter()
        .map(|e| match eval(e.clone(), env) {
            EvalResult::Err(error) => Err(error),
            EvalResult::Expr(expr) => Ok(expr),
            EvalResult::Unit => Ok(e.clone()),
        })
        .collect::<Result<Vec<Rc<Expr>>, String>>();

    total.map_or_else(
        |error| EvalResult::Err(error),
        |xs| {
            let first = &xs[0];
            let result = match op {
                "=" => xs.iter().all(|item| item == first),
                "!=" => xs.iter().any(|item| item != first),
                _ => return EvalResult::Err("Illegal equality operation!".into()),
            };
            match result {
                true => EvalResult::Expr(Expr::symbol("True".into())),
                false => EvalResult::Expr(Expr::symbol("False".into())),
            }
        },
    )
}

/// If statement
/// (if (predicate) (then) (else))
fn if_statement(vals: &[Rc<Expr>], env: &mut Environment) -> EvalResult {
    if vals.len() != 3 {
        return EvalResult::Err(
            "Invalid if statement! Must be 'if (predicate) (then) (else)'".into(),
        );
    }

    let predicate = &vals[0];
    let then = &vals[1];
    let otherwise = &vals[2];
    let test = match eval(predicate.clone(), env) {
        EvalResult::Err(error) => Err(error),
        EvalResult::Expr(expr) => match &*expr {
            Expr::Symbol(s) => match &s[..] {
                "False" => Ok(false),
                _ => Ok(true),
            },
            Expr::List(xs) => match xs[..] {
                [] => Ok(false),
                _ => Ok(true),
            },
            _ => Err("Invalid if statement predicate!".into()),
        },
        EvalResult::Unit => Err("If statement predicate cannot return Unit!".into()),
    };

    test.map_or_else(
        |error| EvalResult::Err(error),
        |test| match test {
            true => eval(then.clone(), env),
            false => eval(otherwise.clone(), env),
        },
    )
}

/// Add a variable to the enviroment
/// (let x expr)
fn add_var_to_env(vals: &[Rc<Expr>], env: &mut Environment) -> EvalResult {
    if vals.len() != 2 {
        return EvalResult::Err("Invalid variable definition! Must be 'let x expr'!".into());
    }

    let var_name = &*vals[0];
    let value = &vals[1];

    if let Expr::Symbol(s) = var_name {
        let check = reserved_words(s);
        if check.is_err() {
            return EvalResult::Err(check.unwrap_err());
        }
    }

    match (var_name, value) {
        (Expr::Symbol(s), expr) => match eval(expr.clone(), env) {
            EvalResult::Err(error) => EvalResult::Err(error),
            EvalResult::Expr(e) => env
                .add_var(s, e)
                .map_or_else(|s| EvalResult::Err(s), |_| EvalResult::Unit),
            EvalResult::Unit => EvalResult::Err("Cannot assign Unit to variable!".into()),
        },
        _ => EvalResult::Err("Invalid variable definition! Must be 'let x expr!".into()),
    }
}

/// Add a function to the enviroment
/// (fn my-func (args) body)
fn add_fun_to_env(vals: &[Rc<Expr>], env: &mut Environment) -> EvalResult {
    if vals.len() != 3 {
        return EvalResult::Err(
            "Invalid function definition! Must be '(fn my-func (args) body)'!".into(),
        );
    }

    let fn_name = &*vals[0];
    let args = &*vals[1];
    let body = &vals[2];

    if let Expr::Symbol(s) = fn_name {
        let check = reserved_words(s);
        if check.is_err() {
            return EvalResult::Err(check.unwrap_err());
        }
    }

    match (&*fn_name, args, body) {
        (Expr::Symbol(fn_name), Expr::List(args), body) => {
            let params = args
                .iter()
                .cloned()
                .map(|e| {
                    if let Expr::Symbol(n) = &*e {
                        Ok(n.to_string())
                    } else {
                        Err("Function arguments must be strings!".into())
                    }
                })
                .collect::<Result<Vec<String>, String>>();

            params.map_or_else(
                |error| EvalResult::Err(error),
                |params| {
                    env.add_fn(fn_name, &params, body.clone())
                        .map_or_else(|error| EvalResult::Err(error), |_| EvalResult::Unit)
                },
            )
        }
        _ => EvalResult::Err(
            "Invalid function definition! Must be '(fn my-func (args) body)'!".into(),
        ),
    }
}

// Reserved words
fn reserved_words(symbol: &str) -> Result<(), String> {
    let reserved = [
        "+", "-", "*", "/", "or", "and", "not", "=", "!=", "if", "let", "fn", "print",
    ];
    if reserved.contains(&symbol) {
        Err("Reserved variable or function name!".into())
    } else {
        Ok(())
    }
}

/// Print function
/// (print 1 2 3)
fn print(vals: &[Rc<Expr>], env: &mut Environment) -> EvalResult {
    if vals.len() < 1 {
        return EvalResult::Err("Missing values in print function'!".into());
    }

    let output = vals
        .iter()
        .map(|e| gen_print_output(e.clone(), env))
        .collect::<Vec<String>>();

    println!("{}", output.join(" "));

    EvalResult::Unit
}

/// Generate output printed to stdout when the user calls print
pub fn gen_print_output(expr: Rc<Expr>, env: &mut Environment) -> String {
    match &*expr {
        Expr::Symbol(s) => match env.lookup(&s) {
            None => format!("{}", s),
            Some((params, e)) if params.is_empty() => gen_print_output(e, env),
            _ => format!("<func-object: {}>", s.to_string()),
        },
        Expr::FNum(n) => format!("{}", n),
        Expr::List(xs) => {
            let output = xs
                .iter()
                .map(|e| gen_print_output(e.clone(), env))
                .collect::<Vec<String>>();
            format!("({})", output.join(" "))
        }
    }
}
