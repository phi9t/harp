use mh2_rs_scheduler::{
    build_schedule, format_minute, plan_jobs, render_report, BackoffPolicy, Calendar, JobSpec,
    ReadyQueue,
};

fn job(id: &str, priority: i32, duration: u32) -> JobSpec {
    JobSpec::new(id, priority, duration)
}

#[test]
fn graph_orders_ready_jobs_by_priority_then_id_and_reports_canonical_errors() {
    let jobs = vec![
        job("package", 6, 10).depends_on(&["compile", "audit"]),
        job("compile", 5, 10).depends_on(&["ingest"]),
        job("audit", 7, 10).depends_on(&["ingest"]),
        job("ingest", 8, 10),
    ];
    assert_eq!(
        plan_jobs(&jobs).unwrap(),
        vec!["ingest", "audit", "compile", "package"]
    );

    let cycle = vec![
        job("b", 1, 1).depends_on(&["c"]),
        job("a", 1, 1).depends_on(&["b"]),
        job("c", 1, 1).depends_on(&["a"]),
    ];
    assert_eq!(
        plan_jobs(&cycle).unwrap_err().to_string(),
        "scheduler:cycle: dependency cycle among a, b, c"
    );

    let missing = vec![job("build", 1, 1).depends_on(&["lint"])];
    assert_eq!(
        plan_jobs(&missing).unwrap_err().to_string(),
        "scheduler:unknown-dependency: job build depends on missing lint"
    );
}

#[test]
fn ready_queue_interleaves_priority_tiers_without_losing_canonical_order() {
    let mut queue = ReadyQueue::new();
    queue.push(job("bulk-b", 9, 5));
    queue.push(job("quick-b", 4, 5));
    queue.push(job("bulk-c", 9, 5));
    queue.push(job("quick-a", 4, 5));
    queue.push(job("bulk-a", 9, 5));

    assert_eq!(
        queue.drain_ids(),
        vec!["bulk-a", "quick-a", "bulk-b", "quick-b", "bulk-c"]
    );
}

#[test]
fn retry_policy_caps_backoff_and_reports_exhaustion_with_scheduler_error() {
    let policy = BackoffPolicy::new(4, 50, 3, 120);
    let events = policy.schedule("compile", &[false, false, true]).unwrap();
    let rows: Vec<(u32, u64, bool)> = events
        .iter()
        .map(|event| (event.attempt, event.elapsed_ms, event.success))
        .collect();
    assert_eq!(rows, vec![(1, 0, false), (2, 50, false), (3, 170, true)]);

    let err = policy
        .schedule("compile", &[false, false, false, false])
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "scheduler:retry-budget-exceeded: job compile failed after 4 attempts"
    );
}

#[test]
fn calendar_places_jobs_inside_minute_windows_and_allows_exact_end_boundary() {
    let jobs = vec![
        job("alpha", 5, 30).window(8 * 60, 9 * 60),
        job("beta", 5, 20).window(8 * 60 + 20, 9 * 60 + 20),
        job("exact", 5, 30).window(8 * 60 + 50, 9 * 60 + 20),
    ];
    let order = vec!["alpha".to_string(), "beta".to_string(), "exact".to_string()];
    let mut calendar = Calendar::new(8 * 60, 10 * 60);
    let placements = calendar.place_all(&jobs, &order).unwrap();
    let rows: Vec<(&str, String, String)> = placements
        .iter()
        .map(|p| {
            (
                p.id.as_str(),
                format_minute(p.start_minute),
                format_minute(p.end_minute),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("alpha", "08:00".to_string(), "08:30".to_string()),
            ("beta", "08:30".to_string(), "08:50".to_string()),
            ("exact", "08:50".to_string(), "09:20".to_string()),
        ]
    );

    let bad = vec![job("bad", 1, 1).window(10, 10)];
    let err = Calendar::new(0, 24 * 60)
        .place_all(&bad, &["bad".to_string()])
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "scheduler:invalid-window: job bad has invalid window 10..10"
    );
}

#[test]
fn scheduler_renders_integrated_report_in_canonical_order() {
    let jobs = vec![
        job("ship", 5, 15)
            .depends_on(&["lint", "test"])
            .window(9 * 60 + 20, 12 * 60)
            .retry(1, 10),
        job("lint", 6, 20)
            .depends_on(&["fetch"])
            .window(8 * 60 + 20, 11 * 60)
            .retry(2, 50),
        job("fetch", 8, 30).window(8 * 60, 10 * 60).retry(2, 25),
        job("test", 7, 25)
            .depends_on(&["fetch"])
            .window(8 * 60 + 20, 11 * 60)
            .retry(3, 40),
    ];
    let report = build_schedule(
        &jobs,
        &[
            ("fetch", vec![true]),
            ("test", vec![false, true]),
            ("lint", vec![true]),
            ("ship", vec![true]),
        ],
    )
    .unwrap();

    assert_eq!(report.order, vec!["fetch", "test", "lint", "ship"]);
    assert_eq!(
        render_report(&report),
        concat!(
            "SCHEDULE v1\n",
            "ORDER fetch -> test -> lint -> ship\n",
            "WINDOWS\n",
            "fetch 08:00-08:30 priority=8 attempts=1 ok\n",
            "test 08:30-08:55 priority=7 attempts=2 ok\n",
            "lint 08:55-09:15 priority=6 attempts=1 ok\n",
            "ship 09:20-09:35 priority=5 attempts=1 ok\n",
            "RETRIES\n",
            "fetch #1 +0ms ok\n",
            "test #1 +0ms failed\n",
            "test #2 +40ms ok\n",
            "lint #1 +0ms ok\n",
            "ship #1 +0ms ok\n",
            "COMPONENTS graph, queue\n"
        )
    );
}
