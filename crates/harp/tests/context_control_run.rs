use harp::context_control::provider::{execute, BoundProviderInvocation, ProviderExecution};
use harp::AppError;

#[test]
fn provider_process_api_consumes_bound_invocation_authority() {
    let execute: fn(BoundProviderInvocation) -> Result<ProviderExecution, AppError> = execute;
    let _ = execute;
}
