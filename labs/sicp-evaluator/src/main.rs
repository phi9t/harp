use sicp_evaluator_lab::Evaluator;

fn main() {
    let expressions: Vec<String> = std::env::args().skip(1).collect();
    let expressions = if expressions.is_empty() {
        vec![
            "(define x 10)".to_string(),
            "(define add-x (lambda (y) (+ x y)))".to_string(),
            "((lambda (x) (add-x 1)) 100)".to_string(),
        ]
    } else {
        expressions
    };

    let mut evaluator = Evaluator::standard();
    for expression in expressions {
        evaluator.reset_metrics();
        match evaluator.eval_source(&expression) {
            Ok(value) => {
                let metrics = evaluator.metrics();
                println!(
                    "{expression} => {value} [dispatches={}, applications={}, frames={}, comparisons={}]",
                    metrics.expression_dispatches,
                    metrics.applications,
                    metrics.frames_visited,
                    metrics.binding_comparisons
                );
            }
            Err(error) => {
                eprintln!("{expression} => error: {error}");
                std::process::exit(1);
            }
        }
    }
}
