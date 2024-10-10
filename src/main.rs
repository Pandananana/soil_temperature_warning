use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use thirtyfour::prelude::*;

fn start_geckodriver() -> Result<Child, std::io::Error> {
    Command::new("geckodriver")
        .arg("--port")
        .arg("4444")
        .spawn()
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // Start geckodriver
    let mut geckodriver = start_geckodriver().expect("Failed to start geckodriver");
    
    // Wait a moment for geckodriver to be ready
    thread::sleep(Duration::from_secs(2));

    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // Your test code here
    driver.goto("https://wikipedia.org").await?;

    // Wait a moment for geckodriver to be ready
    thread::sleep(Duration::from_secs(2));

    // Explicitly close the browser
    driver.quit().await?;

    // Stop geckodriver
    geckodriver.kill().expect("Failed to kill geckodriver");

    Ok(())
}