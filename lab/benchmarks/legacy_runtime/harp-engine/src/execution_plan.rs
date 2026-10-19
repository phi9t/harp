use harp_contracts::TaskGraph;

use crate::{
    validate_graph, CompiledDynamicWorkflow, GraphPolicy, ProjectionPolicy, ValidatedGraph,
    ValidationError,
};

/// The complete graph semantics accepted by the durable Engine boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    graph: TaskGraph,
    graph_policy: GraphPolicy,
    projection_policy: ProjectionPolicy,
}

impl ExecutionPlan {
    pub fn new(
        graph: TaskGraph,
        graph_policy: GraphPolicy,
        projection_policy: ProjectionPolicy,
    ) -> Self {
        Self {
            graph,
            graph_policy,
            projection_policy,
        }
    }

    pub fn validate(self) -> Result<ValidatedExecutionPlan, ValidationError> {
        let validated = validate_graph(self.graph, &self.graph_policy, &self.projection_policy)?;
        Ok(ValidatedExecutionPlan {
            validated,
            graph_policy: self.graph_policy,
            projection_policy: self.projection_policy,
        })
    }
}

impl From<CompiledDynamicWorkflow> for ExecutionPlan {
    fn from(compiled: CompiledDynamicWorkflow) -> Self {
        Self::new(
            compiled.graph,
            compiled.graph_policy,
            compiled.projection_policy,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedExecutionPlan {
    validated: ValidatedGraph,
    graph_policy: GraphPolicy,
    projection_policy: ProjectionPolicy,
}

impl ValidatedExecutionPlan {
    pub fn graph(&self) -> &ValidatedGraph {
        &self.validated
    }

    pub fn graph_policy(&self) -> &GraphPolicy {
        &self.graph_policy
    }

    pub fn projection_policy(&self) -> &ProjectionPolicy {
        &self.projection_policy
    }

    pub fn into_parts(self) -> (ValidatedGraph, GraphPolicy, ProjectionPolicy) {
        (self.validated, self.graph_policy, self.projection_policy)
    }
}
