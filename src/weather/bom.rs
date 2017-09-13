/// Fetches weather from BOM

use weather::Weather;
use weather::WeatherProvider;

use xmltree::Element;
use xmltree::ParseError;

use ftp::FtpStream;
use ftp::FtpError;

use std::error::Error;
use std::io::Cursor;
use std::collections::HashMap;
use std::num::ParseFloatError;
use std::string::FromUtf8Error;

/// Translate FTP error messages
fn translate_ftp_error(data : FtpError) -> String {
    match data {
        FtpError::ConnectionError(err) => {
            err.description().into()
        },
        FtpError::InvalidAddress(err) => {
            err.description().into()
        },
        FtpError::InvalidResponse(str) => {
            str
        },
        FtpError::SecureError(str) => {
            str
        }
    }
}

/// Translates FTP results
fn check_ftp<T>(res : Result<T, FtpError>) -> Result<T, String> {
    match res {
        Ok(result) => {
            Ok(result)
        },
        Err(ftp) => {
            Err(translate_ftp_error(ftp))
        }
    }
}


/// Translate XML error messages
fn translate_xml_error(data : ParseError) -> String {
    match data {
        ParseError::MalformedXml(err) => {
            err.description().into()
        },
        ParseError::CannotParse => {
            "Unable to parse XML".into()
        }
    }
}

/// Translates XML results
fn check_xml<T>(res : Result<T, ParseError>) -> Result<T, String> {
    match res {
        Ok(result) => {
            Ok(result)
        },
        Err(xml) => {
            Err(translate_xml_error(xml))
        }
    }
}

/// Translates parse errors
fn check_parse<T>(res : Result<T, ParseFloatError>) -> Result<T, String> {
    match res {
        Ok(result) => {
            Ok(result)
        },
        Err(err) => {
            Err(err.description().into())
        }
    }
}

/// Translates utf8 parsing errors
fn check_utf<T>(res : Result<T, FromUtf8Error>) -> Result<T, String> {
    match res {
        Ok(result) => {
            Ok(result)
        },
        Err(err) => {
            Err(err.description().into())
        }
    }
}

pub struct BOM;

impl BOM {
    /// Fetches the important metadata from the
    fn fetch_file(&self) -> Result<(Vec<u8>, Vec<u8>), String> {
        let mut ftp = check_ftp(FtpStream::connect("ftp.bom.gov.au:21"))?;
        check_ftp(ftp.login("anonymous", "guest"))?;

        println!("Logged into BOM successfully");

        check_ftp(ftp.cwd("/anon/gen/fwo"))?;

        let data = check_ftp(ftp.simple_retr("IDN10035.xml"))?
            .into_inner().to_owned();

        let data2 = check_ftp(ftp.simple_retr("IDA00101.html"))?
            .into_inner().to_owned();

        check_ftp(ftp.quit())?;

        return Ok((data, data2));
    }
}

impl WeatherProvider for BOM {
    fn get_weather() -> Result<Weather, String> {
        let (xml_des, live_temps) = BOM.fetch_file()?;

        let cursor = Cursor::new(xml_des);
        let element = check_xml(Element::parse(cursor))?;

        let mut recent_store = HashMap::new();
        let mut values = HashMap::new();

        let subset : Result<&Element, String> = element.get_child("forecast")
            .ok_or("\"forecast\" not found".into());
        let subset_elem : Element = subset?.clone();

        for el in subset_elem.children {
            // We now have regions, check to see if they have the information we need
            for period in el.children {
                let index = period.attributes.get("index");
                if index.is_none() {
                    continue;
                }

                let index_value = index.unwrap().parse::<u64>().unwrap();

                for info_kv in period.children {
                    let key = info_kv.attributes.get("type");
                    if key.is_none() {
                        continue;
                    }

                    let key_name = key.unwrap().to_owned();
                    let value = info_kv.text;

                    if value.is_none() {
                        continue;
                    }

                    let value_contents = value.unwrap();

                    if !values.contains_key(&key_name)
                        || recent_store.get(&key_name).unwrap() > &index_value {
                        recent_store.insert(key_name.clone(), index_value);
                        values.insert(key_name, value_contents);
                    }
                }
            }
        }

        // Parse live temperature
        let live_temps = check_utf(String::from_utf8(live_temps))?;

        // Trim off first line (h2 header), and the initial comment
        let mut new_temps = String::new();

        let mut itr = live_temps.lines();
        let _ = itr.next().unwrap();

        for line in itr {
            if line.contains("START OF STANDARD BUREAU FOOTER") {
                break
            }

            new_temps += line;
            new_temps.push('\n');
        }

        // Strip out '&deg;' entities (XML parser doesn't like them)
        new_temps = new_temps.replace("&deg;", "");

        // Now, attempt to parse that
        let live_temps = new_temps.into_bytes();

        let cursor = Cursor::new(live_temps);
        let element = check_xml(Element::parse(cursor))?;

        let tbody : Result<&Element, String> = element.get_child("tbody")
            .ok_or("\"table\" not found".into());
        let tbody_elem : Element = tbody?.clone();

        for elem in tbody_elem.children {
            // Each element here is a city. Find the correct one.
            let children = elem.children;
            let first_child = children[0].clone();
            let second_child = first_child.children[0].clone();
            if second_child.text.unwrap().contains("Canberra") {
                let temp = children[1].clone().text.unwrap();
                values.insert("current_temperature".into(), temp);
            }

        }

        // Grab the info we want
        let precis = values.get("precis").ok_or("Metadata missing weather description")?
            .to_owned();

        let temperature_raw = values.get("current_temperature").ok_or("Missing temperature")?
            .to_owned().parse::<f64>();

        let temperature = check_parse(temperature_raw)?;

        let weather = Weather {
            temperature,
            description : precis
        };

        return Ok(weather);
    }
}
