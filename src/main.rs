use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use thirtyfour::prelude::*;

fn start_geckodriver() -> Result<Child, std::io::Error> {
    let geckodriver = Command::new("geckodriver")
        .arg("--port")
        .arg("4444")
        .spawn();

    // Wait a moment for geckodriver to be ready
    thread::sleep(Duration::from_secs(2));
    geckodriver
}

async fn start_browser() -> WebDriverResult<WebDriver> {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    Ok(driver)
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // Start browser
    let mut geckodriver = start_geckodriver().expect("Failed to start geckodriver");
    let driver = start_browser().await?;

    // Your test code here
    driver.goto("https://weathermodels-plant.dlbr.dk/(S(gzy2tilppcazluhtlh4hleft))/default.aspx").await?;

    // Wait a moment for geckodriver to be ready
    thread::sleep(Duration::from_secs(2));

    // Explicitly close the browser
    driver.quit().await?;

    // Stop geckodriver
    geckodriver.kill().expect("Failed to kill geckodriver");

    Ok(())
}