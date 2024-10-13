use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use thirtyfour::prelude::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use chrono::Local;


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

async fn stop_browser_gecko(webdriver: WebDriver, mut geckodriver: Child) {
    // Explicitly close the browser
    webdriver.quit().await.expect("Failed to close browser");

    // Stop geckodriver
    geckodriver.kill().expect("Failed to kill geckodriver");
}

fn send_email(recepient: &str, subject: &str, body: &str) {
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set");
    let smtp_server = env::var("SMTP_SERVER").expect("SMTP_SERVER not set");
    let from_email = env::var("FROM_EMAIL").expect("FROM_EMAIL not set");

    // Create the email
    let email = Message::builder()
        .from(from_email.parse().expect("Couldn't parse from_email variable"))
        .to(recepient.parse().expect("Couldn't parse to_email variable"))
        .subject(subject)
        .body(String::from(body)).expect("Couldn't create email body");

    // Create the SMTP transport
    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpTransport::relay(&smtp_server).expect("Couldn't create mailer")
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {e:?}"),
    }
}

async fn get_soil_temp(driver: &WebDriver) -> Result<String, Box<dyn std::error::Error>>{
    // Your test code here
    driver.goto("https://weathermodels-plant.dlbr.dk/(S(gzy2tilppcazluhtlh4hleft))/default.aspx").await?;

    let latitude = env::var("LATITUDE").expect("LATITUDE not set");
    let longitude = env::var("LONGITUDE").expect("LONGTITUDE not set");

    // Clear and then insert latitude
    let input_lat = driver.find(By::Id("txtCoordinateLat")).await?;
    input_lat.clear().await?;
    input_lat.send_keys(latitude).await?;

    // Clear and then insert longitude
    let input_long = driver.find(By::Id("txtCoordinateLon")).await?;
    input_long.clear().await?;
    input_long.send_keys(longitude).await?;

    // Get date
    let date_formatted = Local::now().format("%d-%m-%Y").to_string();

    // Clear and then insert from date
    let input_date_from = driver.find(By::Id("datePickFrom_dateInput")).await?;
    input_date_from.clear().await?;
    input_date_from.send_keys(&date_formatted).await?;

    // Clear and then insert to date
    let input_date_to = driver.find(By::Id("datePickTo_dateInput")).await?;
    input_date_to.clear().await?;
    input_date_to.send_keys(&date_formatted).await?;

    // Select soil temperature reading
    driver.find(By::Id("chkParameters_3")).await?.click().await?;

    // Click update
    driver.find(By::Id("Button1")).await?.click().await?;

    // Get soil temp data
    let soil_temp = driver.find(By::XPath("//*[@id='GridView1']/tbody/tr[2]/td[7]")).await?.text().await?;
    
    Ok(soil_temp)
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // Start browser
    let geckodriver = start_geckodriver().expect("Failed to start geckodriver");
    let driver = start_browser().await.expect("Failed to start browser");

    match get_soil_temp(&driver).await {
        Ok(soil_temp) => println!("{}", soil_temp),
        Err(error) => println!("Error getting soil temp: {}", error)
    }


    stop_browser_gecko(driver, geckodriver).await;

    Ok(())
}