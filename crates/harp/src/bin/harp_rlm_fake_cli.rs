#[cfg(feature = "test-cli-fixture")]
fn main() {
    std::process::exit(harp_cli_test_support::run_fake_cli());
}

#[cfg(not(feature = "test-cli-fixture"))]
fn main() {
    eprintln!("harp_rlm_fake_cli requires the test-cli-fixture feature");
    std::process::exit(70);
}
