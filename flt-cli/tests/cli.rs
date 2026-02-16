use std::path::Path;
use std::process::Command;

use cucumber::then;
use cucumber::when;
use cucumber::World;

#[derive(Debug, Default, World)]
pub struct CliWorld {
    pub status: Option<i32>,
    pub output: Option<String>,
}

#[when(regex = r#"^the command `(.+)` is run$"#)]
async fn the_command_is_run(world: &mut CliWorld, cmd: String) {
    println!("the_command_is_run: {}", cmd);
    let parts = cmd.split_whitespace().collect::<Vec<&str>>();
    assert!(!parts.is_empty(), "No command provided");
    println!("run_command: {}", cmd);
    let mut args: Vec<&str> = parts[1..].to_vec();
    let executable = if parts[0] == "flt" {
        args.insert(0, "--");
        args.insert(0, "run");
        "cargo"
    } else {
        parts[0]
    };
    println!("Running {} {}", executable, args.join(" "));
    match Command::new(executable).args(args).output() {
        Ok(output) => {
            world.status = Some(output.status.code().unwrap());
            let output_str = String::from_utf8(output.stdout).unwrap();
            println!("Output: {}", output_str);
            world.output = Some(output_str);
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
