// allow-unwrap-in-tests in clippy.toml doesn't work in `tests/`
#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::Command;

use cucumber::then;
use cucumber::when;
use cucumber::World;

#[derive(Debug, Default, World)]
pub struct CliWorld {
    pub status: Option<i32>,
    pub output: Option<String>,
    pub error_output: Option<String>,
}

#[when(regex = r#"^the command `(.+)` is run$"#)]
async fn the_command_is_run(world: &mut CliWorld, cmd: String) {
    let parts = shell_words::split(&cmd).unwrap();
    assert!(!parts.is_empty(), "No command provided");
    let args: Vec<&str> = parts[1..].iter().map(String::as_str).collect();
    // Run the built binary directly: `cargo run` costs about 200 ms a scenario
    let executable = if parts[0] == "flt" {
        env!("CARGO_BIN_EXE_flt")
    } else {
        parts[0].as_str()
    };
    println!("Running {} {}", executable, args.join(" "));
    match Command::new(executable).args(args).output() {
        Ok(output) => {
            world.status = Some(output.status.code().unwrap());
            let output_str = String::from_utf8(output.stdout).unwrap();
            println!("Output: {}", output_str);
            world.output = Some(output_str);
            world.error_output = Some(String::from_utf8(output.stderr).unwrap());
        }
        Err(e) => {
            panic!("Failed to run command: {}", e);
        }
    };
}

#[then(expr = "it should exit with status code {int}")]
async fn it_should_exit_with_status(world: &mut CliWorld, status: i32) {
    println!("status: {:?}", status);
    assert!(world.status.unwrap() == status);
}

#[then(expr = r"the output should contain {string}")]
async fn the_output_should_contain(world: &mut CliWorld, expected: String) {
    assert!(world.output.is_some(), "No output");
    assert!(
        world.output.as_ref().unwrap().contains(&expected),
        "Output does not contain {}",
        expected
    );
}

#[then(expr = r"the output should not contain {string}")]
async fn the_output_should_not_contain(world: &mut CliWorld, unexpected: String) {
    let output = world.output.as_ref().expect("No output");
    assert!(
        !output.contains(&unexpected),
        "Output contains {}: {}",
        unexpected,
        output
    );
}

#[then(expr = r"the error output should contain {string}")]
async fn the_error_output_should_contain(world: &mut CliWorld, expected: String) {
    let error_output = world.error_output.as_ref().expect("No error output");
    assert!(
        error_output.contains(&expected),
        "Error output does not contain {}: {}",
        expected,
        error_output
    );
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_target(false)
        .format_timestamp_secs()
        .target(env_logger::Target::Stdout)
        .init();
    log::info!("Running CLI tests");

    let features = Path::new(env!("CARGO_MANIFEST_DIR")).join("features/cli.feature");
    CliWorld::run(features).await;
}
