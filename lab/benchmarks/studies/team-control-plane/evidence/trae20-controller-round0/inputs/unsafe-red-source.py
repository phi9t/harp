"""Unsafe public-boundary baseline for a preimplementation RED; never CLI launch."""
def run_case(*, case_path, grant_path, grant_revision, operation_id, repository, output_directory, executor, read_committed=None):
    executor({'operation_id':operation_id})
    return {'status':'executed','executed':True}
