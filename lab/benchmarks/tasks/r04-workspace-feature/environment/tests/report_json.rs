use report::render_report;

#[test]
fn default_workspace_feature_produces_json_report() {
    assert_eq!(render_report(7, "ready"), "{\"id\":7,\"payload\":\"ready\"}");
}

#[test]
fn json_report_escapes_quotes_and_backslashes() {
    assert_eq!(
        render_report(9, "path \"C:\\tmp\""),
        "{\"id\":9,\"payload\":\"path \\\"C:\\\\tmp\\\"\"}"
    );
}
