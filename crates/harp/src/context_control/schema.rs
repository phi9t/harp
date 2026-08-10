use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AppError;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Trae,
    Codex,
}

impl fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trae => "trae",
            Self::Codex => "codex",
        })
    }
}

impl FromStr for ProviderId {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "trae" => Ok(Self::Trae),
            "codex" => Ok(Self::Codex),
            _ => Err(AppError::invalid_input(
                "context_control.provider_id",
                format!("unknown provider ID: {value}"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowId {
    CiRepair,
    CodeReview,
    DependencyUpdate,
    GeneralCoding,
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CiRepair => "ci_repair",
            Self::CodeReview => "code_review",
            Self::DependencyUpdate => "dependency_update",
            Self::GeneralCoding => "general_coding",
        })
    }
}

impl FromStr for WorkflowId {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ci_repair" => Ok(Self::CiRepair),
            "code_review" => Ok(Self::CodeReview),
            "dependency_update" => Ok(Self::DependencyUpdate),
            "general_coding" => Ok(Self::GeneralCoding),
            _ => Err(AppError::invalid_input(
                "context_control.workflow_id",
                format!("unknown workflow ID: {value}"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkflowChoice {
    Auto,
    Workflow(WorkflowId),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum ContextItemKind {
    RepositoryInvariant,
    DiagnosticRule,
    WorkflowStep,
    VerificationRecipe,
    AntiPattern,
    ToolUsageRule,
    ArchitectureFact,
    ReviewPreference,
    Example,
    Exception,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_ids_have_stable_wire_names() {
        assert_eq!(
            serde_json::to_string(&ProviderId::Trae).unwrap(),
            "\"trae\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderId::Codex).unwrap(),
            "\"codex\""
        );
        assert_eq!(ProviderId::Trae.to_string(), "trae");
        assert_eq!("codex".parse::<ProviderId>().unwrap(), ProviderId::Codex);
        assert_eq!(
            "unknown".parse::<ProviderId>().unwrap_err().code(),
            "context_control.provider_id"
        );
    }

    #[test]
    fn workflow_ids_have_stable_wire_names() {
        let cases = [
            (WorkflowId::CiRepair, "ci_repair"),
            (WorkflowId::CodeReview, "code_review"),
            (WorkflowId::DependencyUpdate, "dependency_update"),
            (WorkflowId::GeneralCoding, "general_coding"),
        ];

        for (workflow, wire_name) in cases {
            assert_eq!(
                serde_json::to_string(&workflow).unwrap(),
                format!("\"{wire_name}\"")
            );
            assert_eq!(workflow.to_string(), wire_name);
            assert_eq!(wire_name.parse::<WorkflowId>().unwrap(), workflow);
        }
        assert_eq!(
            "unknown".parse::<WorkflowId>().unwrap_err().code(),
            "context_control.workflow_id"
        );
    }

    #[test]
    fn context_item_kinds_have_stable_wire_names() {
        let cases = [
            (ContextItemKind::RepositoryInvariant, "repository_invariant"),
            (ContextItemKind::DiagnosticRule, "diagnostic_rule"),
            (ContextItemKind::WorkflowStep, "workflow_step"),
            (ContextItemKind::VerificationRecipe, "verification_recipe"),
            (ContextItemKind::AntiPattern, "anti_pattern"),
            (ContextItemKind::ToolUsageRule, "tool_usage_rule"),
            (ContextItemKind::ArchitectureFact, "architecture_fact"),
            (ContextItemKind::ReviewPreference, "review_preference"),
            (ContextItemKind::Example, "example"),
            (ContextItemKind::Exception, "exception"),
        ];

        for (kind, wire_name) in cases {
            assert_eq!(
                serde_json::to_string(&kind).unwrap(),
                format!("\"{wire_name}\"")
            );
        }
    }
}
