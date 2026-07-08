// allow-unwrap-in-tests in clippy.toml doesn't work in `tests/`
#![allow(clippy::unwrap_used)]

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use cucumber::gherkin::Step;
use cucumber::then;
use cucumber::when;
use cucumber::World;

#[derive(Debug, Default, World)]
pub struct ReplWorld {
    pub output: Option<String>,
    pub last_output: Option<std::process::Output>,
}

#[when(regex = r#"^the REPL is run and the user types:$"#)]
async fn the_repl_is_run_and_the_user_types(world: &mut ReplWorld, step: &Step) {
    let input = step.docstring.as_ref().expect("Step requires a docstring");
    let mut child = if let Ok(flt_path) = std::env::var("CARGO_BIN_EXE_flt") {
        Command::new(flt_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn flt REPL")
    } else {
        Command::new("cargo")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["run", "--"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn flt REPL")
    };

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        for line in input.trim().lines() {
            stdin.write_all(line.trim().as_bytes()).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed waiting for flt");
    world.last_output = Some(output);
    let last = world.last_output.as_ref().unwrap();
    world.output = Some(format!(
        "{}{}",
        String::from_utf8(last.stdout.clone()).unwrap(),
        String::from_utf8(last.stderr.clone()).unwrap()
    ));
}

#[then(regex = r#"^the command should succeed$"#)]
async fn the_command_should_succeed(world: &mut ReplWorld) {
    let output = world
        .last_output
        .as_ref()
        .expect("No command output; use 'the REPL is run and the user types' first");
    assert!(
        output.status.success(),
        "Command failed with exit code {:?}:\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[then(expr = r"the output should contain {string}")]
async fn the_output_should_contain(world: &mut ReplWorld, expected: String) {
    assert!(world.output.is_some(), "No output");
    assert!(
        world.output.as_ref().unwrap().contains(&expected),
        "Output does not contain {expected}",
    );
}

#[then(expr = r"the output should not contain {string}")]
async fn the_output_should_not_contain(world: &mut ReplWorld, unexpected: String) {
    assert!(world.output.is_some(), "No output");
    assert!(
        !world.output.as_ref().unwrap().contains(&unexpected),
        "Output should not contain {unexpected}",
    );
}

#[tokio::main]
async fn main() {
    let features = Path::new(env!("CARGO_MANIFEST_DIR")).join("features/repl");
    ReplWorld::run(features).await;
}
