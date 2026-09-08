mod bindings;
mod cancel;
mod diagnostic;
mod dynamic_workflow;
mod execution_plan;
mod inspection;
mod output;
mod policy;
mod projection;
mod recovery;
mod scheduler;
mod validate;
mod workflow_v2;

pub use cancel::EngineControl;
pub use diagnostic::{ValidationDiagnostic, ValidationError};
pub use dynamic_workflow::{
    compile_dynamic_workflow, CompiledDynamicWorkflow, DynamicWorkflowCompileError,
    DynamicWorkflowCompileOptions,
};
pub use execution_plan::{ExecutionPlan, ValidatedExecutionPlan};
pub use inspection::{explain_run, InspectionAction, WorkflowInspection};
pub use output::decode_result_envelope;
pub use policy::{ApprovedArtifact, GraphPolicy};
pub use projection::{
    project_child_context, BoundedProjectedJson, ProjectedChildContext, ProjectedInput,
    ProjectionError, ProjectionPolicy, ProjectionRequest,
};
pub use recovery::{recovery_action, RecoveryAction};
pub use scheduler::{
    CrashPoint, Engine, EngineConfig, EngineError, RunExecutionSpec, RunSummary, WallClock,
};
pub use validate::{role_name, validate_graph, workspace_mode_name, ValidatedGraph};

pub use workflow_v2::{validate_workflow_v2, ValidatedWorkflowV2, WorkflowValidationError};

pub use bindings::{
    resolve_workflow_inputs, AcceptedTaskOutputs, OutputSchemaRegistry, ResolvedWorkflowInput,
    ResolvedWorkflowInputs, WorkloadOutcome,
};

mod job_admission;
pub use job_admission::validate_command_backend_authority;
