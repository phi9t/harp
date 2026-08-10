fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(harp_cli_test_support::run_fake_cli() as u8)
}
