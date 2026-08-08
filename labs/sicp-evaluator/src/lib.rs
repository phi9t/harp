use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Number(i64),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
}

#[derive(Clone, Copy)]
pub struct Primitive {
    name: &'static str,
    function: fn(&[Value]) -> Result<Value, EvalError>,
}

#[derive(Clone)]
pub struct Closure {
    parameters: Vec<String>,
    body: Expr,
    environment: EnvironmentRef,
}

#[derive(Clone)]
pub enum Value {
    Number(i64),
    Bool(bool),
    Symbol(String),
    List(Vec<Value>),
    Primitive(Primitive),
    Closure(Rc<Closure>),
}

impl fmt::Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(number) => write!(formatter, "{number}"),
            Self::Bool(true) => write!(formatter, "#t"),
            Self::Bool(false) => write!(formatter, "#f"),
            Self::Symbol(symbol) => write!(formatter, "{symbol}"),
            Self::List(values) => {
                write!(formatter, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, " ")?;
                    }
                    write!(formatter, "{value}")?;
                }
                write!(formatter, ")")
            }
            Self::Primitive(primitive) => write!(formatter, "<primitive:{}>", primitive.name),
            Self::Closure(closure) => {
                write!(formatter, "<closure:({})>", closure.parameters.join(" "))
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Symbol(left), Self::Symbol(right)) => left == right,
            (Self::List(left), Self::List(right)) => left == right,
            (Self::Primitive(left), Self::Primitive(right)) => left.name == right.name,
            (Self::Closure(left), Self::Closure(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl Eq for Value {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalError(String);

impl EvalError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for EvalError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub expression_dispatches: u64,
    pub applications: u64,
    pub binding_comparisons: u64,
    pub frames_visited: u64,
}

type EnvironmentRef = Rc<Environment>;

struct Environment {
    // A vector deliberately mirrors SICP's linear frame scan instead of optimizing it.
    bindings: RefCell<Vec<(String, Value)>>,
    parent: Option<EnvironmentRef>,
}

impl Environment {
    fn new(parent: Option<EnvironmentRef>) -> EnvironmentRef {
        Rc::new(Self {
            bindings: RefCell::new(Vec::new()),
            parent,
        })
    }

    fn define(&self, name: impl Into<String>, value: Value) {
        let name = name.into();
        let mut bindings = self.bindings.borrow_mut();
        if let Some((_, existing)) = bindings.iter_mut().find(|(key, _)| key == &name) {
            *existing = value;
        } else {
            bindings.push((name, value));
        }
    }

    fn lookup(&self, name: &str, metrics: &mut Metrics) -> Result<Value, EvalError> {
        metrics.frames_visited += 1;
        for (candidate, value) in self.bindings.borrow().iter() {
            metrics.binding_comparisons += 1;
            if candidate == name {
                return Ok(value.clone());
            }
        }
        match &self.parent {
            Some(parent) => parent.lookup(name, metrics),
            None => Err(EvalError::new(format!("unbound variable: {name}"))),
        }
    }
}

pub struct Evaluator {
    global: EnvironmentRef,
    metrics: Metrics,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::standard()
    }
}

impl Evaluator {
    pub fn standard() -> Self {
        let global = Environment::new(None);
        for primitive in [
            Primitive {
                name: "+",
                function: add,
            },
            Primitive {
                name: "-",
                function: subtract,
            },
            Primitive {
                name: "*",
                function: multiply,
            },
            Primitive {
                name: "=",
                function: numeric_equal,
            },
        ] {
            global.define(primitive.name, Value::Primitive(primitive));
        }
        Self {
            global,
            metrics: Metrics::default(),
        }
    }

    pub fn eval_source(&mut self, source: &str) -> Result<Value, EvalError> {
        let expression = parse(source)?;
        self.eval(&expression, self.global.clone())
    }

    pub fn metrics(&self) -> Metrics {
        self.metrics
    }

    pub fn reset_metrics(&mut self) {
        self.metrics = Metrics::default();
    }

    fn eval(&mut self, expression: &Expr, environment: EnvironmentRef) -> Result<Value, EvalError> {
        self.metrics.expression_dispatches += 1;
        match expression {
            Expr::Number(number) => Ok(Value::Number(*number)),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::Symbol(name) => environment.lookup(name, &mut self.metrics),
            Expr::List(items) if items.is_empty() => Err(EvalError::new("cannot evaluate ()")),
            Expr::List(items) => match items.first() {
                Some(Expr::Symbol(keyword)) if keyword == "quote" => self.eval_quote(items),
                Some(Expr::Symbol(keyword)) if keyword == "if" => self.eval_if(items, environment),
                Some(Expr::Symbol(keyword)) if keyword == "define" => {
                    self.eval_define(items, environment)
                }
                Some(Expr::Symbol(keyword)) if keyword == "lambda" => {
                    self.eval_lambda(items, environment)
                }
                Some(Expr::Symbol(keyword)) if keyword == "begin" => {
                    self.eval_sequence(&items[1..], environment)
                }
                _ => {
                    let procedure = self.eval(&items[0], environment.clone())?;
                    let mut arguments = Vec::with_capacity(items.len().saturating_sub(1));
                    for operand in &items[1..] {
                        arguments.push(self.eval(operand, environment.clone())?);
                    }
                    self.apply(procedure, arguments)
                }
            },
        }
    }

    fn eval_quote(&self, items: &[Expr]) -> Result<Value, EvalError> {
        require_length("quote", items, 2)?;
        Ok(datum_to_value(&items[1]))
    }

    fn eval_if(&mut self, items: &[Expr], environment: EnvironmentRef) -> Result<Value, EvalError> {
        require_length("if", items, 4)?;
        let predicate = self.eval(&items[1], environment.clone())?;
        let selected = if is_truthy(&predicate) {
            &items[2]
        } else {
            &items[3]
        };
        self.eval(selected, environment)
    }

    fn eval_define(
        &mut self,
        items: &[Expr],
        environment: EnvironmentRef,
    ) -> Result<Value, EvalError> {
        require_length("define", items, 3)?;
        let Expr::Symbol(name) = &items[1] else {
            return Err(EvalError::new("define requires a symbol name"));
        };
        let value = self.eval(&items[2], environment.clone())?;
        environment.define(name.clone(), value);
        Ok(Value::Symbol("ok".to_string()))
    }

    fn eval_lambda(&self, items: &[Expr], environment: EnvironmentRef) -> Result<Value, EvalError> {
        require_length("lambda", items, 3)?;
        let Expr::List(parameter_expressions) = &items[1] else {
            return Err(EvalError::new("lambda parameters must be a list"));
        };
        let parameters = parameter_expressions
            .iter()
            .map(|expression| match expression {
                Expr::Symbol(name) => Ok(name.clone()),
                _ => Err(EvalError::new("lambda parameters must be symbols")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Closure(Rc::new(Closure {
            parameters,
            body: items[2].clone(),
            environment,
        })))
    }

    fn eval_sequence(
        &mut self,
        expressions: &[Expr],
        environment: EnvironmentRef,
    ) -> Result<Value, EvalError> {
        if expressions.is_empty() {
            return Err(EvalError::new("begin requires at least one expression"));
        }
        let mut result = Value::Bool(false);
        for expression in expressions {
            result = self.eval(expression, environment.clone())?;
        }
        Ok(result)
    }

    fn apply(&mut self, procedure: Value, arguments: Vec<Value>) -> Result<Value, EvalError> {
        self.metrics.applications += 1;
        match procedure {
            Value::Primitive(primitive) => (primitive.function)(&arguments),
            Value::Closure(closure) => {
                if closure.parameters.len() != arguments.len() {
                    return Err(EvalError::new(format!(
                        "closure expected {} arguments, received {}",
                        closure.parameters.len(),
                        arguments.len()
                    )));
                }
                let call_environment = Environment::new(Some(closure.environment.clone()));
                for (parameter, argument) in closure.parameters.iter().zip(arguments) {
                    call_environment.define(parameter.clone(), argument);
                }
                self.eval(&closure.body, call_environment)
            }
            other => Err(EvalError::new(format!(
                "attempted to apply non-procedure: {other}"
            ))),
        }
    }
}

fn require_length(form: &str, items: &[Expr], expected: usize) -> Result<(), EvalError> {
    if items.len() == expected {
        Ok(())
    } else {
        Err(EvalError::new(format!(
            "{form} expected {} parts, received {}",
            expected,
            items.len()
        )))
    }
}

fn datum_to_value(expression: &Expr) -> Value {
    match expression {
        Expr::Number(number) => Value::Number(*number),
        Expr::Bool(value) => Value::Bool(*value),
        Expr::Symbol(symbol) => Value::Symbol(symbol.clone()),
        Expr::List(items) => Value::List(items.iter().map(datum_to_value).collect()),
    }
}

fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Bool(false))
}

fn numbers(arguments: &[Value], operation: &str) -> Result<Vec<i64>, EvalError> {
    arguments
        .iter()
        .map(|value| match value {
            Value::Number(number) => Ok(*number),
            other => Err(EvalError::new(format!(
                "{operation} expected numbers, received {other}"
            ))),
        })
        .collect()
}

fn add(arguments: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Number(numbers(arguments, "+")?.iter().sum()))
}

fn multiply(arguments: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Number(numbers(arguments, "*")?.iter().product()))
}

fn subtract(arguments: &[Value]) -> Result<Value, EvalError> {
    let values = numbers(arguments, "-")?;
    let Some((first, rest)) = values.split_first() else {
        return Err(EvalError::new("- expected at least one argument"));
    };
    let result = if rest.is_empty() {
        -*first
    } else {
        rest.iter()
            .fold(*first, |accumulator, value| accumulator - value)
    };
    Ok(Value::Number(result))
}

fn numeric_equal(arguments: &[Value]) -> Result<Value, EvalError> {
    let values = numbers(arguments, "=")?;
    if values.len() != 2 {
        return Err(EvalError::new(format!(
            "= expected 2 arguments, received {}",
            values.len()
        )));
    }
    Ok(Value::Bool(values[0] == values[1]))
}

pub fn parse(source: &str) -> Result<Expr, EvalError> {
    let tokens = tokenize(source);
    let mut cursor = 0;
    let expression = parse_expression(&tokens, &mut cursor)?;
    if cursor != tokens.len() {
        return Err(EvalError::new("unexpected trailing input"));
    }
    Ok(expression)
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in source.chars() {
        match character {
            '(' | ')' | '\'' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                tokens.push(character.to_string());
            }
            character if character.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_expression(tokens: &[String], cursor: &mut usize) -> Result<Expr, EvalError> {
    let token = tokens
        .get(*cursor)
        .ok_or_else(|| EvalError::new("unexpected end of input"))?;
    *cursor += 1;
    match token.as_str() {
        "(" => {
            let mut items = Vec::new();
            while tokens.get(*cursor).map(String::as_str) != Some(")") {
                if *cursor >= tokens.len() {
                    return Err(EvalError::new("unterminated list"));
                }
                items.push(parse_expression(tokens, cursor)?);
            }
            *cursor += 1;
            Ok(Expr::List(items))
        }
        ")" => Err(EvalError::new("unexpected )")),
        "'" => Ok(Expr::List(vec![
            Expr::Symbol("quote".to_string()),
            parse_expression(tokens, cursor)?,
        ])),
        "#t" => Ok(Expr::Bool(true)),
        "#f" => Ok(Expr::Bool(false)),
        atom => match atom.parse::<i64>() {
            Ok(number) => Ok(Expr::Number(number)),
            Err(_) => Ok(Expr::Symbol(atom.to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Evaluator, Value};

    fn evaluate(source: &str) -> Value {
        Evaluator::standard().eval_source(source).unwrap()
    }

    #[test]
    fn evaluates_nested_combinations() {
        assert_eq!(evaluate("(+ 1 (* 2 3))"), Value::Number(7));
    }

    #[test]
    fn closures_use_the_environment_captured_at_definition() {
        let mut evaluator = Evaluator::standard();
        evaluator.eval_source("(define x 10)").unwrap();
        evaluator
            .eval_source("(define add-x (lambda (y) (+ x y)))")
            .unwrap();

        let value = evaluator
            .eval_source("((lambda (x) (add-x 1)) 100)")
            .unwrap();

        assert_eq!(value, Value::Number(11));
    }

    #[test]
    fn if_evaluates_only_the_selected_branch() {
        assert_eq!(evaluate("(if #t 7 missing)"), Value::Number(7));
    }

    #[test]
    fn quote_turns_program_syntax_into_data() {
        assert_eq!(
            evaluate("'(a (+ 1 2))"),
            Value::List(vec![
                Value::Symbol("a".to_string()),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Number(1),
                    Value::Number(2),
                ]),
            ])
        );
    }

    #[test]
    fn deep_binding_records_frame_and_binding_search_cost() {
        let mut evaluator = Evaluator::standard();
        let value = evaluator
            .eval_source("((lambda (a) ((lambda (b) (+ a b)) 2)) 3)")
            .unwrap();

        assert_eq!(value, Value::Number(5));
        assert!(evaluator.metrics().frames_visited >= 4);
        assert!(evaluator.metrics().binding_comparisons >= 4);
    }

    #[test]
    fn rejects_wrong_closure_arity() {
        let error = Evaluator::standard()
            .eval_source("((lambda (x) x) 1 2)")
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "closure expected 1 arguments, received 2"
        );
    }
}
