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

pub struct BOM;

impl BOM {
    /// Fetches the important metadata from the
    fn fetch_file(&self) -> Result<Vec<u8>, String> {
        let mut ftp = check_ftp(FtpStream::connect("ftp.bom.gov.au:21"))?;
        check_ftp(ftp.login("anonymous", "guest"))?;

        println!("Logged into BOM successfully");

        check_ftp(ftp.cwd("/anon/gen/fwo"))?;

        let data = check_ftp(ftp.simple_retr("IDN10035.xml"))?
            .into_inner().to_owned();

        check_ftp(ftp.quit())?;

        return Ok(data);
    }
}

impl WeatherProvider for BOM {
    fn get_weather() -> Result<Weather, String> {
        let file = BOM.fetch_file()?;

        let cursor = Cursor::new(file);
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

        let precis = values.get("precis").ok_or("Metadata missing weather description")?
            .to_owned();

        // TODO: Proper current temperature
        let temperature_raw = values.get("air_temperature_maximum").ok_or("Missing temperature")?
            .to_owned().parse::<f64>();

        let temperature = check_parse(temperature_raw)?;

        let weather = Weather {
            temperature,
            description : precis
        };

        return Ok(weather);
    }
}
