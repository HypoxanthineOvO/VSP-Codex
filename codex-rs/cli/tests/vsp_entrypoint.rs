use anyhow::Result;
use predicates::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use tempfile::TempDir;

fn vsp_codex_command() -> Result<assert_cmd::Command> {
    Ok(assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin(
        "vsp-codex",
    )?))
}

fn toml_string_literal(value: &str) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn spawn_responses_sse_server() -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    thread::spawn(move || {
        for _ in 0..4 {
            let Ok((mut stream, _addr)) = listener.accept() else {
                return;
            };
            let Ok(cloned_stream) = stream.try_clone() else {
                return;
            };
            let mut reader = BufReader::new(cloned_stream);
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or_default() == 0 {
                    return;
                }
                if line == "\r\n" {
                    break;
                }
                let Some((name, value)) = line.split_once(':') else {
                    continue;
                };
                if name.eq_ignore_ascii_case("content-length")
                    && let Ok(value) = value.trim().parse()
                {
                    content_length = value;
                }
            }
            if content_length > 0 {
                let mut body = vec![0; content_length];
                let _ = std::io::Read::read_exact(&mut reader, &mut body);
            }

            let body = concat!(
                "event: response.created\n",
                "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp-vsp-fixture\"}}\n\n",
                "event: response.completed\n",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp-vsp-fixture\",\"usage\":{\"input_tokens\":0,\"input_tokens_details\":null,\"output_tokens\":0,\"output_tokens_details\":null,\"total_tokens\":0}}}\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    Ok(format!("http://{addr}/v1"))
}

fn vsp_codex_exec_with_mock_server(codex_home: &std::path::Path) -> Result<assert_cmd::Command> {
    let base_url = spawn_responses_sse_server()?;
    let mut cmd = vsp_codex_command()?;
    cmd.env("CODEX_HOME", codex_home)
        .env("OPENAI_API_KEY", "dummy")
        .arg("exec")
        .arg("--skip-git-repo-check")
        .arg("-c")
        .arg(format!(
            "model_providers.mock={{ name = \"mock\", base_url = {}, env_key = \"OPENAI_API_KEY\", wire_api = \"responses\", request_max_retries = 0, stream_max_retries = 0 }}",
            toml_string_literal(&base_url)?
        ))
        .arg("-c")
        .arg("model_provider=\"mock\"")
        .arg("-C")
        .arg(codex_utils_cargo_bin::repo_root()?);
    Ok(cmd)
}

fn write_config_toml(codex_home: &std::path::Path, contents: &str) -> Result<()> {
    std::fs::write(codex_home.join("config.toml"), contents)?;
    Ok(())
}

#[test]
fn vsp_codex_version_entrypoint_is_available() -> Result<()> {
    vsp_codex_command()?
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("vsp-codex "));

    Ok(())
}

#[test]
fn vsp_codex_exec_defaults_to_yolo_permissions_without_explicit_config() -> Result<()> {
    let codex_home = TempDir::new()?;

    let output = vsp_codex_exec_with_mock_server(codex_home.path())?
        .arg("default permissions regression")
        .output()?;

    assert!(output.status.success(), "exec failed: {output:?}");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("approval: never"),
        "stderr missing default approval policy: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: danger-full-access"),
        "stderr missing default YOLO sandbox: {stderr}"
    );

    Ok(())
}

#[test]
fn vsp_codex_exec_defaults_to_yolo_permissions_with_approval_only_config() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_config_toml(codex_home.path(), "approval_policy = \"on-request\"\n")?;

    let output = vsp_codex_exec_with_mock_server(codex_home.path())?
        .arg("default permissions regression")
        .output()?;

    assert!(output.status.success(), "exec failed: {output:?}");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("approval: never"),
        "stderr missing headless approval policy: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: danger-full-access"),
        "stderr missing default YOLO sandbox with approval-only config: {stderr}"
    );

    Ok(())
}

#[test]
fn vsp_codex_exec_preserves_config_toml_sandbox_override() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_config_toml(codex_home.path(), "sandbox_mode = \"workspace-write\"\n")?;

    let output = vsp_codex_exec_with_mock_server(codex_home.path())?
        .arg("default permissions regression")
        .output()?;

    assert!(output.status.success(), "exec failed: {output:?}");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("approval: never"),
        "stderr missing headless approval policy: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: workspace-write"),
        "stderr missing config.toml sandbox override: {stderr}"
    );

    Ok(())
}

#[test]
fn vsp_codex_exec_preserves_explicit_sandbox_override() -> Result<()> {
    let codex_home = TempDir::new()?;

    let output = vsp_codex_exec_with_mock_server(codex_home.path())?
        .arg("--sandbox")
        .arg("workspace-write")
        .arg("default permissions regression")
        .output()?;

    assert!(output.status.success(), "exec failed: {output:?}");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("approval: never"),
        "stderr missing headless approval policy: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: workspace-write"),
        "stderr missing explicit sandbox override: {stderr}"
    );

    Ok(())
}

#[test]
fn vsp_codex_help_uses_vsp_codex_usage() -> Result<()> {
    vsp_codex_command()?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: vsp-codex [OPTIONS] [PROMPT]",
        ))
        .stdout(predicate::str::contains(
            "vsp-codex [OPTIONS] <COMMAND> [ARGS]",
        ))
        .stdout(predicate::str::contains("Usage: codex [OPTIONS]").not())
        .stdout(predicate::str::contains("cloud").not())
        .stdout(predicate::str::contains("login").not())
        .stdout(predicate::str::contains("logout").not())
        .stdout(predicate::str::contains("remote-control").not())
        .stdout(predicate::str::contains("update").not());

    Ok(())
}

#[test]
fn official_service_cli_commands_are_disabled() -> Result<()> {
    for args in [
        &["login"][..],
        &["logout"][..],
        &["cloud", "list"][..],
        &["remote-control"][..],
        &["update"][..],
        &["app-server", "daemon", "enable-remote-control"][..],
        &["app-server", "daemon", "bootstrap", "--remote-control"][..],
    ] {
        vsp_codex_command()?
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "This official Codex service is disabled in VSP Codex.",
            ));
    }

    Ok(())
}

#[test]
fn vsp_codex_completion_uses_vsp_codex_command_name() -> Result<()> {
    vsp_codex_command()?
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vsp-codex"))
        .stdout(predicate::str::contains("complete -F _codex codex").not());

    Ok(())
}
