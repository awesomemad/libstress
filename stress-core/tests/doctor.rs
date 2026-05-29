use stress_core::doctor::run_checks;

#[test]
fn doctor_runs() {
    let report = run_checks();
    assert!(!report.checks.is_empty());
}
