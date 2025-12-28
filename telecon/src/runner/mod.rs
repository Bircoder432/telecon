use crate::parser::{HandlerConfig, Service};
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

pub async fn run_custom_handler(handler: &HandlerConfig) {
    let mut cmd = Command::new(&handler.exec);
    cmd.args(&handler.args);

    match cmd.output().await {
        Ok(output) => {
            println!(
                "Custom handler `{}` output:\n{}",
                handler.name,
                String::from_utf8_lossy(&output.stdout)
            );
            if !output.stderr.is_empty() {
                eprintln!(
                    "Custom handler `{}` error:\n{}",
                    handler.name,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to run custom handler `{}`: {}", handler.name, e);
        }
    }
}
