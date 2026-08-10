use super::workflow::{RoutingRules, WorkflowPackage};
use super::{WorkflowChoice, WorkflowId};
use crate::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteDecision {
    pub selected: WorkflowId,
    pub explicit: bool,
    pub considered: Vec<RouteConsideration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteConsideration {
    pub workflow: WorkflowId,
    pub score: i32,
    pub matched_rule_ids: Vec<String>,
    pub rejection_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateEvaluation {
    workflow: WorkflowId,
    score: i32,
    matched_rule_ids: Vec<String>,
    suppression_reason: Option<String>,
}

#[derive(Clone, Copy)]
struct RoutingConfig<'a> {
    workflow: WorkflowId,
    rules: &'a RoutingRules,
}

struct Rule<'a> {
    id: &'a str,
    score: i32,
    all_phrases: &'a [&'a str],
    any_phrases: &'a [&'a str],
    none_phrases: &'a [&'a str],
}

struct RuleMatches<'a> {
    score: i32,
    rule_ids: Vec<&'a str>,
}

pub fn route(task: &str, choice: WorkflowChoice) -> Result<RouteDecision, AppError> {
    let packages = WorkflowId::ALL
        .into_iter()
        .map(|workflow| WorkflowPackage::builtin(workflow).map(|package| (workflow, package)))
        .collect::<Result<Vec<_>, _>>()?;
    let configs = packages
        .iter()
        .map(|(workflow, package)| RoutingConfig {
            workflow: *workflow,
            rules: package.routing(),
        })
        .collect::<Vec<_>>();

    Ok(evaluate_route(task, choice, &configs))
}

fn evaluate_route(
    task: &str,
    choice: WorkflowChoice,
    configs: &[RoutingConfig<'_>],
) -> RouteDecision {
    let normalized_task = normalize_task(task);
    let mut evaluations = configs
        .iter()
        .map(|config| evaluate_workflow(&normalized_task, *config))
        .collect::<Vec<_>>();
    evaluations.sort_by_key(|evaluation| evaluation.workflow);
    let (selected, explicit) = match choice {
        WorkflowChoice::Auto => (select_auto(&evaluations), false),
        WorkflowChoice::Workflow(workflow) => (workflow, true),
    };

    let considered = build_trace(evaluations, selected, explicit);
    RouteDecision {
        selected,
        explicit,
        considered,
    }
}

fn normalize_task(task: &str) -> String {
    let lowercase = task.to_lowercase();
    let mut normalized = String::with_capacity(lowercase.len());
    let mut pending_space = false;

    for character in lowercase.chars() {
        if character.is_ascii_whitespace() {
            pending_space = !normalized.is_empty();
        } else {
            if pending_space {
                normalized.push(' ');
                pending_space = false;
            }
            normalized.push(character);
        }
    }

    normalized
}

fn evaluate_workflow(normalized_task: &str, config: RoutingConfig<'_>) -> CandidateEvaluation {
    let negative_phrases = config
        .rules
        .negative_phrases
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    if let Some(phrase) = negative_phrases
        .iter()
        .find(|phrase| normalized_task.contains(**phrase))
    {
        return CandidateEvaluation {
            workflow: config.workflow,
            score: 0,
            matched_rule_ids: Vec::new(),
            suppression_reason: Some(format!("suppressed by negative phrase {phrase:?}")),
        };
    }

    let mut score = 0;
    let mut matched_rule_ids = Vec::new();
    for (index, phrase) in config.rules.positive_phrases.iter().enumerate() {
        let id = format!("{}.positive.{index}", config.workflow);
        let all_phrases = [phrase.as_str()];
        let rule = Rule {
            id: &id,
            score: 1,
            all_phrases: &all_phrases,
            any_phrases: &[],
            none_phrases: &negative_phrases,
        };
        let rules = [rule];
        let matched = evaluate_rules(normalized_task, &rules);
        score += matched.score;
        matched_rule_ids.extend(matched.rule_ids.into_iter().map(str::to_owned));
    }

    CandidateEvaluation {
        workflow: config.workflow,
        score,
        matched_rule_ids,
        suppression_reason: None,
    }
}

fn evaluate_rules<'a>(task: &str, rules: &'a [Rule<'a>]) -> RuleMatches<'a> {
    let mut score = 0;
    let mut rule_ids = Vec::new();

    for rule in rules {
        let all_match = rule.all_phrases.iter().all(|phrase| task.contains(phrase));
        let any_match = rule.any_phrases.is_empty()
            || rule.any_phrases.iter().any(|phrase| task.contains(phrase));
        let none_match = rule
            .none_phrases
            .iter()
            .all(|phrase| !task.contains(phrase));
        if all_match && any_match && none_match {
            score += rule.score;
            rule_ids.push(rule.id);
        }
    }

    RuleMatches { score, rule_ids }
}

fn select_auto(evaluations: impl AsRef<[CandidateEvaluation]>) -> WorkflowId {
    let eligible = evaluations
        .as_ref()
        .iter()
        .filter(|evaluation| {
            evaluation.workflow != WorkflowId::GeneralCoding
                && evaluation.suppression_reason.is_none()
                && evaluation.score > 0
        })
        .collect::<Vec<_>>();
    let Some(top_score) = eligible.iter().map(|evaluation| evaluation.score).max() else {
        return WorkflowId::GeneralCoding;
    };
    let mut top = eligible
        .into_iter()
        .filter(|evaluation| evaluation.score == top_score);
    let selected = top.next().expect("positive top score has a candidate");
    if top.next().is_some() {
        WorkflowId::GeneralCoding
    } else {
        selected.workflow
    }
}

fn build_trace(
    evaluations: Vec<CandidateEvaluation>,
    selected: WorkflowId,
    explicit: bool,
) -> Vec<RouteConsideration> {
    let top_score = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.workflow != WorkflowId::GeneralCoding
                && evaluation.suppression_reason.is_none()
        })
        .map(|evaluation| evaluation.score)
        .max()
        .unwrap_or(0);
    let top_count = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.workflow != WorkflowId::GeneralCoding
                && evaluation.suppression_reason.is_none()
                && evaluation.score == top_score
        })
        .count();
    let selected_score = evaluations
        .iter()
        .find(|evaluation| evaluation.workflow == selected)
        .map_or(0, |evaluation| evaluation.score);

    let mut considered = evaluations
        .into_iter()
        .map(|evaluation| {
            let rejection_reason = rejection_reason(
                &evaluation,
                selected,
                selected_score,
                explicit,
                top_score,
                top_count,
            );
            RouteConsideration {
                workflow: evaluation.workflow,
                score: evaluation.score,
                matched_rule_ids: evaluation.matched_rule_ids,
                rejection_reason,
            }
        })
        .collect::<Vec<_>>();
    considered.sort_by_key(|consideration| consideration.workflow);
    considered
}

fn rejection_reason(
    evaluation: &CandidateEvaluation,
    selected: WorkflowId,
    selected_score: i32,
    explicit: bool,
    top_score: i32,
    top_count: usize,
) -> Option<String> {
    if evaluation.workflow == selected {
        return None;
    }
    if explicit {
        return Some(format!("explicit workflow choice selected {selected}"));
    }
    if let Some(reason) = &evaluation.suppression_reason {
        return Some(reason.clone());
    }
    if evaluation.workflow == WorkflowId::GeneralCoding {
        return Some(format!(
            "fallback not needed because {selected} had unique positive score {selected_score}"
        ));
    }
    if top_score <= 0 {
        return Some(format!(
            "score {} was not positive, so fallback was selected",
            evaluation.score
        ));
    }
    if evaluation.score == top_score && top_count > 1 {
        return Some(format!(
            "top positive score {top_score} was tied, so fallback was selected"
        ));
    }
    if selected == WorkflowId::GeneralCoding {
        return Some(format!(
            "score {} did not qualify for automatic selection",
            evaluation.score
        ));
    }
    Some(format!(
        "score {} was below selected score {selected_score}",
        evaluation.score
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_the_required_task_matrix() {
        let cases = [
            ("fix the tests failing in CI", WorkflowId::CiRepair),
            ("review this diff and find bugs", WorkflowId::CodeReview),
            (
                "upgrade dependency serde and refresh lockfile",
                WorkflowId::DependencyUpdate,
            ),
            (
                "add a bounded parser for this config",
                WorkflowId::GeneralCoding,
            ),
        ];

        for (task, expected) in cases {
            let decision = route(task, WorkflowChoice::Auto).unwrap();
            assert_eq!(decision.selected, expected, "task: {task}");
            assert!(!decision.explicit);
        }
    }

    #[test]
    fn explicit_choice_wins_regardless_of_task() {
        let decision = route(
            "fix the tests failing in CI",
            WorkflowChoice::Workflow(WorkflowId::CodeReview),
        )
        .unwrap();

        assert_eq!(decision.selected, WorkflowId::CodeReview);
        assert!(decision.explicit);
        assert_eq!(
            decision
                .considered
                .iter()
                .find(|entry| entry.workflow == WorkflowId::CodeReview)
                .unwrap()
                .rejection_reason,
            None
        );
    }

    #[test]
    fn tied_positive_top_score_falls_back_to_general_coding() {
        let decision = route("tests failing code review", WorkflowChoice::Auto).unwrap();

        assert_eq!(decision.selected, WorkflowId::GeneralCoding);
        let tied = decision
            .considered
            .iter()
            .filter(|entry| {
                matches!(
                    entry.workflow,
                    WorkflowId::CiRepair | WorkflowId::CodeReview
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(tied.len(), 2);
        assert!(tied.iter().all(|entry| entry.score == 1));
        assert!(tied.iter().all(|entry| entry
            .rejection_reason
            .as_deref()
            .unwrap()
            .contains("tied")));
    }

    #[test]
    fn zero_and_negative_top_scores_use_the_fallback() {
        let zero = select_auto(vec![
            evaluation(WorkflowId::CiRepair, 0, true),
            evaluation(WorkflowId::CodeReview, 0, true),
            evaluation(WorkflowId::DependencyUpdate, 0, true),
            evaluation(WorkflowId::GeneralCoding, 0, true),
        ]);
        assert_eq!(zero, WorkflowId::GeneralCoding);

        let negative = select_auto(vec![
            evaluation(WorkflowId::CiRepair, -1, true),
            evaluation(WorkflowId::CodeReview, -2, true),
            evaluation(WorkflowId::DependencyUpdate, -3, true),
            evaluation(WorkflowId::GeneralCoding, 0, true),
        ]);
        assert_eq!(negative, WorkflowId::GeneralCoding);
    }

    #[test]
    fn negative_phrase_suppresses_candidate_before_tie_handling() {
        let decision = route(
            "review this diff and find bugs while tests failing",
            WorkflowChoice::Auto,
        )
        .unwrap();

        assert_eq!(decision.selected, WorkflowId::CodeReview);
        let ci_repair = decision
            .considered
            .iter()
            .find(|entry| entry.workflow == WorkflowId::CiRepair)
            .unwrap();
        assert_eq!(ci_repair.score, 0);
        assert!(ci_repair.matched_rule_ids.is_empty());
        assert!(ci_repair
            .rejection_reason
            .as_deref()
            .unwrap()
            .contains("negative phrase"));
    }

    #[test]
    fn rule_matching_honors_all_any_none_and_adds_scores() {
        let task = normalize_task("Alpha beta gamma");
        let rules = [
            Rule {
                id: "all-and-any",
                score: 4,
                all_phrases: &["alpha", "beta"],
                any_phrases: &["missing", "gamma"],
                none_phrases: &["blocked"],
            },
            Rule {
                id: "empty-any",
                score: -1,
                all_phrases: &["alpha"],
                any_phrases: &[],
                none_phrases: &[],
            },
            Rule {
                id: "missing-all",
                score: 20,
                all_phrases: &["alpha", "delta"],
                any_phrases: &[],
                none_phrases: &[],
            },
            Rule {
                id: "forbidden",
                score: 30,
                all_phrases: &["alpha"],
                any_phrases: &[],
                none_phrases: &["gamma"],
            },
        ];

        let matched = evaluate_rules(&task, &rules);
        assert_eq!(matched.score, 3);
        assert_eq!(matched.rule_ids, ["all-and-any", "empty-any"]);
    }

    #[test]
    fn trace_is_complete_stably_sorted_and_explains_every_unselected_workflow() {
        let decision = route("review this diff and find bugs", WorkflowChoice::Auto).unwrap();

        assert_eq!(decision.considered.len(), WorkflowId::ALL.len());
        assert_workflow_order(&decision.considered);
        for entry in &decision.considered {
            if entry.workflow == decision.selected {
                assert_eq!(entry.rejection_reason, None);
                assert!(!entry.matched_rule_ids.is_empty());
            } else {
                assert!(
                    entry
                        .rejection_reason
                        .as_deref()
                        .is_some_and(|reason| !reason.is_empty()),
                    "missing rejection reason for {}",
                    entry.workflow
                );
            }
        }
    }

    #[test]
    fn pure_evaluator_sorts_deliberately_shuffled_inputs() {
        let fixtures = routing_fixtures();
        let shuffled = [
            RoutingConfig {
                workflow: WorkflowId::GeneralCoding,
                rules: &fixtures[3],
            },
            RoutingConfig {
                workflow: WorkflowId::DependencyUpdate,
                rules: &fixtures[2],
            },
            RoutingConfig {
                workflow: WorkflowId::CodeReview,
                rules: &fixtures[1],
            },
            RoutingConfig {
                workflow: WorkflowId::CiRepair,
                rules: &fixtures[0],
            },
        ];

        let decision = evaluate_route(
            "review this diff and find bugs",
            WorkflowChoice::Auto,
            &shuffled,
        );

        assert_eq!(decision.selected, WorkflowId::CodeReview);
        assert_workflow_order(&decision.considered);
    }

    #[test]
    fn normalization_uses_unicode_lowercase_and_collapses_ascii_whitespace() {
        assert_eq!(
            normalize_task("  TESTS\tFAILING \n Ä  parser\r\n"),
            "tests failing ä parser"
        );
        assert_eq!(
            route("  TESTS\tFAILING \n Ä  ", WorkflowChoice::Auto)
                .unwrap()
                .selected,
            WorkflowId::CiRepair
        );
    }

    #[test]
    fn routing_source_smoke_check_rejects_direct_side_effect_apis() {
        let source = include_str!("routing.rs");
        let test_boundary = ["#[cfg", "(test)]"].concat();
        let production = source
            .split_once(&test_boundary)
            .expect("routing module must keep tests behind cfg(test)")
            .0;

        assert_eq!(find_forbidden_apis(production), Vec::<String>::new());

        let forbidden_examples = [
            ["use std::", "fs;"].concat(),
            ["use std::", "net::TcpStream;"].concat(),
            ["use std::", "process::Command;"].concat(),
            ["Command::", "new(\"git\")"].concat(),
            ["req", "west::Client::new()"].concat(),
            ["Http", "Client::new()"].concat(),
            ["Model", "Client::generate()"].concat(),
        ];
        for example in forbidden_examples {
            assert!(
                !find_forbidden_apis(&example).is_empty(),
                "purity guard missed synthetic side-effect API: {example}"
            );
        }
    }

    #[test]
    fn pure_evaluator_is_repeatable_with_in_memory_routing_data() {
        let fixtures = routing_fixtures();
        let configs = routing_configs(&fixtures);
        let first = evaluate_route(
            "upgrade dependency serde and refresh lockfile",
            WorkflowChoice::Auto,
            &configs,
        );

        for _ in 0..32 {
            assert_eq!(
                evaluate_route(
                    "upgrade dependency serde and refresh lockfile",
                    WorkflowChoice::Auto,
                    &configs,
                ),
                first
            );
        }
    }

    fn routing_fixtures() -> [RoutingRules; 4] {
        [
            routing_rules(&["tests failing"], &["review this diff"]),
            routing_rules(&["review this diff", "find bugs"], &["upgrade dependency"]),
            routing_rules(
                &["upgrade dependency", "refresh lockfile"],
                &["review this diff"],
            ),
            routing_rules(&[], &[]),
        ]
    }

    fn routing_configs(fixtures: &[RoutingRules; 4]) -> [RoutingConfig<'_>; 4] {
        [
            RoutingConfig {
                workflow: WorkflowId::CiRepair,
                rules: &fixtures[0],
            },
            RoutingConfig {
                workflow: WorkflowId::CodeReview,
                rules: &fixtures[1],
            },
            RoutingConfig {
                workflow: WorkflowId::DependencyUpdate,
                rules: &fixtures[2],
            },
            RoutingConfig {
                workflow: WorkflowId::GeneralCoding,
                rules: &fixtures[3],
            },
        ]
    }

    fn routing_rules(positive: &[&str], negative: &[&str]) -> RoutingRules {
        RoutingRules {
            schema_version: "test".to_owned(),
            positive_phrases: positive.iter().map(|phrase| (*phrase).to_owned()).collect(),
            negative_phrases: negative.iter().map(|phrase| (*phrase).to_owned()).collect(),
        }
    }

    fn assert_workflow_order(considered: &[RouteConsideration]) {
        assert!(considered
            .windows(2)
            .all(|pair| pair[0].workflow < pair[1].workflow));
    }

    fn evaluation(workflow: WorkflowId, score: i32, eligible: bool) -> CandidateEvaluation {
        CandidateEvaluation {
            workflow,
            score,
            matched_rule_ids: Vec::new(),
            suppression_reason: (!eligible).then(|| "suppressed".to_owned()),
        }
    }

    fn find_forbidden_apis(source: &str) -> Vec<String> {
        [
            ["std::", "fs"].concat(),
            ["std::{", "fs"].concat(),
            ["std::", "net"].concat(),
            ["std::{", "net"].concat(),
            ["std::", "process"].concat(),
            ["std::{", "process"].concat(),
            ["Command::", "new"].concat(),
            ["Command::", "spawn"].concat(),
            [".", "spawn("].concat(),
            ["req", "west::"].concat(),
            ["u", "req::"].concat(),
            ["hy", "per::"].concat(),
            ["Http", "Client"].concat(),
            ["Client::", "new"].concat(),
            ["Client::", "builder"].concat(),
            ["Model", "Client"].concat(),
            ["model_", "client"].concat(),
            [".", "generate("].concat(),
            [".", "complete("].concat(),
            [".", "chat("].concat(),
        ]
        .into_iter()
        .filter(|pattern| source.contains(pattern))
        .collect()
    }
}
