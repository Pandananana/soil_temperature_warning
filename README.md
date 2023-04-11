# Soil Temperature Monitoring

This Python script uses Selenium library to scrape soil temperature data from a Danish website and send email notifications based on predefined temperature thresholds. It is intended to be used for monitoring soil temperature in a specific location and sending email alerts when the temperature falls below certain thresholds.

<br>

## Getting started
1. Install Python 3 on your local machine, if not already installed.
2. Install the required libraries using pip:

        pip install selenium webdriver_manager pandas
3. Create a `sender.txt` file, which needs to contain 1 line with 3 comma separated strings. The first string should have the email sender address, the second string should have the email sender password, and lastly the third string should have the smpt url of your mailserver. There shouldn't be any spaces, and the last entry shouldn't have a comma.
4. Create an `emails.txt` file, which needs to contain 2 lines, the first line should have the admin(s) email addresses, and the second line should have the general subscribers' emails. They need to be comma separated (except for the last entry) and with no spaces.

## Usage
Run the script using Python to start monitoring the soil temperature and sending email notifications:

        python3 soil_temperature_monitor.py

The script will scrape the soil temperature data from the website, compare it with the predefined thresholds, and send email notifications if the temperature falls below the thresholds.

Note: If you want to disable sending email notifications, you can set the `send_email` variable to `False` in the script.

## Contributing
Contributions are welcome! If you have any suggestions, improvements, or bug fixes, please create a pull request.