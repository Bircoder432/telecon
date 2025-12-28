use crate::parser::Service;
use tokio::process::Command;

pub async fn run_service(service: &Service) {
    let mut cmd = Command::new(&service.command);
    cmd.args(&service.args);

    match cmd.output().await {
        Ok(output) => {
            println!(
                "Service `{}` output:\n{}",
                service.name,
                String::from_utf8_lossy(&output.stdout)
            );
            if !output.stderr.is_empty() {
                eprintln!(
                    "Service `{}` error:\n{}",
                    service.name,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to start service `{}`: {}", service.name, e);
        }
    }
}
