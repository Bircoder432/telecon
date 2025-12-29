use tokio::process::Command;

pub struct CommandRunner;

impl CommandRunner {
    pub async fn run(command: &str, args: &[String]) -> Result<String, String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if !output.stderr.is_empty() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
