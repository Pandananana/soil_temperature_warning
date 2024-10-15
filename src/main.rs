use chrono::{DateTime, Utc, Local};
use serde::{Deserialize, Serialize};
use std::{
    process::{Command, Child},
    thread,
    time::Duration,
    env,
    fs,
    path::Path,
};
use thirtyfour::prelude::*;
use lettre::{
    Message, SmtpTransport, Transport,
    transport::smtp::authentication::Credentials,
};
use anyhow::{Context, Result};

async fn get_current_temperature() -> Result<f32> {
    // Start geckodriver
    let mut geckodriver = start_geckodriver().context("Failed to start geckodriver")?;

    // Use a closure to ensure browser is always closed
    let result = (|| async {
        let driver = start_browser().await.context("Failed to start browser")?;

        // Navigate to the weather page
        driver.goto("https://weathermodels-plant.dlbr.dk/(S(gzy2tilppcazluhtlh4hleft))/default.aspx")
            .await
            .context("Failed to navigate to weather page")?;

        // Get coordinates from environment variables
        let latitude = env::var("LATITUDE").context("LATITUDE not set")?;
        let longitude = env::var("LONGITUDE").context("LONGITUDE not set")?;

        // Helper function to clear and set input
        async fn set_input(driver: &WebDriver, id: &str, value: &str) -> WebDriverResult<()> {
            let input = driver.find(By::Id(id)).await?;
            input.clear().await?;
            input.send_keys(value).await?;
            Ok(())
        }

        // Set latitude and longitude
        set_input(&driver, "txtCoordinateLat", &latitude).await?;
        set_input(&driver, "txtCoordinateLon", &longitude).await?;

        // Set dates
        let date_formatted = Local::now().format("%d-%m-%Y").to_string();
        set_input(&driver, "datePickFrom_dateInput", &date_formatted).await?;
        set_input(&driver, "datePickTo_dateInput", &date_formatted).await?;

        // Select soil temperature reading and update
        driver.find(By::Id("chkParameters_3")).await?.click().await?;
        driver.find(By::Id("Button1")).await?.click().await?;

        // Get soil temperature data
        let soil_temp = driver.find(By::XPath("//*[@id='GridView1']/tbody/tr[2]/td[7]"))
            .await?
            .text()
            .await?;

        parse_comma_float(&soil_temp).context("Failed to parse soil temperature")
    })().await;

    // Ensure geckodriver is stopped
    geckodriver.kill().context("Failed to kill geckodriver")?;

    // Print the result
    println!("Current temperature: {:?}", result);

    result
}

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
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    Ok(driver)
}

fn parse_comma_float(s: &str) -> Result<f32> {
    s.replace(',', ".")
        .parse()
        .context("Failed to parse float")
}

async fn send_email(recipient: &str, subject: &str, body: &str) -> Result<()> {
    let smtp_username = env::var("SMTP_USERNAME").context("SMTP_USERNAME not set")?;
    let smtp_password = env::var("SMTP_PASSWORD").context("SMTP_PASSWORD not set")?;
    let smtp_server = env::var("SMTP_SERVER").context("SMTP_SERVER not set")?;
    let from_email = env::var("FROM_EMAIL").context("FROM_EMAIL not set")?;

    // Create the email
    let email = Message::builder()
        .from(from_email.parse().context("Invalid from_email")?)
        .to(recipient.parse().context("Invalid recipient email")?)
        .subject(subject)
        .body(String::from(body))
        .context("Failed to create email")?;

    // Create the SMTP transport
    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpTransport::relay(&smtp_server)
        .context("Failed to create SMTP transport")?
        .credentials(creds)
        .build();

    // Send the email
    // mailer.send(&email).context("Failed to send email")?;
    println!("Email sent successfully!");
    Ok(())
}

async fn send_error_email(error: &anyhow::Error) -> Result<()> {
    let admin_email = env::var("ADMIN_EMAIL").context("ADMIN_EMAIL not set")?;
    let subject = "Error in Temperature Monitor".to_string();
    let body = format!("An error occurred in the Temperature Monitor:\n\n{:#}", error);
    
    send_email(&admin_email, &subject, &body).await
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
enum WarningLevel {
    None,
    Low,    // Below 2°C
    Medium, // Below 1°C
    High,   // Below 0°C
}

#[derive(Serialize, Deserialize, Debug)]
struct TemperatureMonitor {
    current_warning_level: WarningLevel,
    last_email_sent: Option<DateTime<Utc>>,
}

impl TemperatureMonitor {
    fn new() -> Self {
        println!("Creating new TemperatureMonitor");
        TemperatureMonitor {
            current_warning_level: WarningLevel::None,
            last_email_sent: None,
        }
    }

    async fn daily_check(&mut self) -> Result<()> {
        print!("Daily check: ");
        let current_temp = get_current_temperature().await.context("Failed to get current temperature")?;
        let new_warning_level = self.determine_warning_level(current_temp);

        if current_temp >= 5.0 {
            println!("Temperature is above 5°C, warnings reset");
            self.current_warning_level = WarningLevel::None;
        } else if new_warning_level > self.current_warning_level {
            self.send_warning_email(&new_warning_level).await?;
            self.current_warning_level = new_warning_level;
        }
        Ok(())
    }

    fn determine_warning_level(&self, temperature: f32) -> WarningLevel {
        if temperature <= 0.0 {
            WarningLevel::High
        } else if temperature <= 1.0 {
            WarningLevel::Medium
        } else if temperature <= 2.0 {
            WarningLevel::Low
        } else {
            WarningLevel::None
        }
    }

    async fn send_warning_email(&mut self, level: &WarningLevel) -> Result<()> {
        // Implement email sending logic here
        println!("Sending warning email for level: {:?}", level);
        // You would call the send_email function here with appropriate parameters
        // send_email("recipient@example.com", "Temperature Warning", &format!("Warning level: {:?}", level)).await?;
        self.last_email_sent = Some(Utc::now());
        Ok(())
    }

    fn save_state(&self) -> Result<()> {
        println!("Saving state");
        let json = serde_json::to_string(self).context("Failed to serialize data")?;
        fs::write("data.json", json).context("Failed to write data to file")?;
        Ok(())
    }

    fn load_state() -> Result<Self> {
        println!("Loading state");
        if !Path::new("data.json").exists() {
            return Ok(TemperatureMonitor::new());
        }

        let json = fs::read_to_string("data.json").context("Failed to read data file")?;
        let monitor: TemperatureMonitor = serde_json::from_str(&json).context("Failed to deserialize data")?;

        println!("Loaded state: {:?}", monitor);
        Ok(monitor)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let result = async {
        let mut monitor = TemperatureMonitor::load_state().context("Failed to load state")?;
        monitor.daily_check().await.context("Failed to perform daily check")?;
        monitor.save_state().context("Failed to save state")?;
        Ok(())
    }.await;

    if let Err(err) = &result {
        eprintln!("An error occurred: {:?}", err);
        if let Err(email_err) = send_error_email(err).await {
            eprintln!("Failed to send error email: {:?}", email_err);
        }
    }

    result
}