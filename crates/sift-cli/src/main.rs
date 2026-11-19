//! `sift`: the command line (MASTER_PLAN section 10).

fn main() {
    match sift_cli::run_cli(std::env::args_os()) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(2);
        }
    }
}
