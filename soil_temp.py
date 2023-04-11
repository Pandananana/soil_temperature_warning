from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import TimeoutException
from selenium.webdriver.firefox.service import Service as FirefoxService
from selenium.webdriver import FirefoxOptions
from webdriver_manager.firefox import GeckoDriverManager

import time
import locale
from datetime import date

from email.message import EmailMessage
from email.header import Header
from email.utils import formataddr
import ssl
import smtplib
import csv
import pandas as pd


## ------ Editable Variables ------ ##
send_email = True

# Open email receiver file
with open("emails.txt", newline='') as f:
  reader = csv.reader(f)
  admin_receiver = next(reader)
  email_receiver = next(reader)

# Open email sender info file
with open("sender.txt", newline='') as f:
  reader = csv.reader(f)
  sender_list = next(reader)
  email_sender = sender_list[0]
  email_password = sender_list[1]
  smpt_url = sender_list[2]

try:
    # Set up Firefox options
    opts = FirefoxOptions()
    opts.add_argument("--headless")
    driver = webdriver.Firefox(service=FirefoxService(GeckoDriverManager().install()),options=opts)

    # Define and format inputs
    locale.setlocale(locale.LC_NUMERIC, "en_DK.UTF-8")
    gps_lat = 55.646850736063776
    gps_long = 12.638347013144639
    today = date.today()
    soil_temp = 0
    
    # Get Previous Data
    csv_path = "temperatur.csv"
    df = pd.read_csv(csv_path, sep=",")
    temp_last = df.iloc[0, 0]
    warning_1 = df.iloc[0, 1]
    warning_2 = df.iloc[0, 2]
    warning_3 = df.iloc[0, 3]
    warning_4 = df.iloc[0, 4]

    # Setup Email
    email_name = "Kolonihave Jordmåler"
    em = EmailMessage()
    em["From"] = formataddr((str(Header(email_name, "utf-8")), email_sender))
    em["To"] = ", ".join(email_receiver)
    em.set_content("""
    Temperaturen er målt dagligt i en dybde på 10cm.

    Denne mail kan ikke besvares. 
    Kontakt aw@winstondesign.dk for spørgsmål
    """)

    ## ------ Start Browser Bot ------ ##
    driver.get(
        "https://weathermodels-plant.dlbr.dk/(S(gzy2tilppcazluhtlh4hleft))/default.aspx")
    driver.implicitly_wait(30)
    time.sleep(0.2)
    input_lat = driver.find_element(By.ID, "txtCoordinateLat")
    input_lat.clear()
    input_lat.send_keys(locale.format_string("%.8f", gps_lat))
    input_long = driver.find_element(By.ID, "txtCoordinateLon")
    input_long.clear()
    input_long.send_keys(locale.format_string("%.8f", gps_long))
    input_data_from = driver.find_element(By.ID, "datePickFrom_dateInput")
    input_data_from.clear()
    input_data_from.send_keys(today.strftime("%d-%m-%Y"))
    input_data_to = driver.find_element(By.ID, "datePickTo_dateInput")
    input_data_to.clear()
    input_data_to.send_keys(today.strftime("%d-%m-%Y"))
    radio_soil = driver.find_element(By.ID, "chkParameters_3").click()
    btn_update = driver.find_element(By.ID, "Button1").click()

    # Waits for data to be visible on page
    while (time.time() < time.time()+30):
        soil_temp = driver.find_element(
            By.XPATH, "//*[@id='GridView1']/tbody/tr[2]/td[7]").text
        if (soil_temp != "Nan"):
            break
    soil_temp_float = locale.atof(soil_temp)
    df.iloc[0, 0] = soil_temp_float
    driver.quit()

    ## ------ Soil Temperature Checks ------ ##
    if soil_temp_float == 0:
        em["Subject"] = "Ingen måling"
    elif (temp_last > 1.5) and (soil_temp_float <= 1.5) and warning_1:
        em["Subject"] = "ADVARSEL - Temperatur: {:.02f} °C".format(
            soil_temp_float)
        df.iloc[0, 1] = 0
    elif (temp_last > 1.0) and (soil_temp_float <= 1.0) and warning_2:
        em["Subject"] = "ADVARSEL - Temperatur: {:.02f} °C".format(
            soil_temp_float)
        df.iloc[0, 2] = 0
    elif (temp_last > 0.5) and (soil_temp_float <= 0.5) and warning_3:
        em["Subject"] = "ADVARSEL - Temperatur: {:.02f} °C".format(
            soil_temp_float)
        df.iloc[0, 3] = 0
    elif (temp_last > 0) and (soil_temp_float <= 0) and warning_4:
        em["Subject"] = "ADVARSEL - Temperatur: {:.02f} °C".format(
            soil_temp_float)
        df.iloc[0, 4] = 0
    else:
        if soil_temp_float >= 2:
            if warning_1 == 0:
                em["Subject"] = "Temperaturen er normal"
                df.iloc[0, 1] = 1
                df.iloc[0, 2] = 1
                df.iloc[0, 3] = 1
                df.iloc[0, 4] = 1
            else:
                df.iloc[0, 1] = 1
                df.iloc[0, 2] = 1
                df.iloc[0, 3] = 1
                df.iloc[0, 4] = 1
                df.to_csv(csv_path, sep=",", header=True, index=False)
                print("Temperatur: {:.02f} °C".format(soil_temp_float))
                raise SystemExit
        else:
            df.to_csv(csv_path, sep=",", header=True, index=False)
            print("Temperatur: {:.02f} °C".format(soil_temp_float))
            raise SystemExit

    ## ------ Send Email ------ ##
    context = ssl.create_default_context()
    if send_email:
        with smtplib.SMTP_SSL(smpt_url, 465, context=context) as smtp:
            smtp.login(email_sender, email_password)
            smtp.sendmail(email_sender, email_receiver, em.as_string())
    else:
        print(em)
    df.to_csv(csv_path, sep=",", header=True, index=False)


except Exception as _ex:
    email_name = "Kolonihave Jordmåler"
    em = EmailMessage()
    em["Subject"] = "ERROR"
    em["From"] = formataddr((str(Header(email_name, "utf-8")), email_sender))
    if len(admin_receiver)>1:
        em["To"] = ", ".join(admin_receiver)
    else:
        em["To"] = admin_receiver[0]
    em.set_content(repr(_ex))
    context = ssl.create_default_context()
    if send_email:
        with smtplib.SMTP_SSL(smpt_url, 465, context=context) as smtp:
            smtp.login(email_sender, email_password)
            smtp.sendmail(email_sender, email_receiver, em.as_string())
    else:
        print(em)
