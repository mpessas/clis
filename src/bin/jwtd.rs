use base64_url;
use serde_json;
use std::env;
use std::error::Error;
use std::process;
use std::str;

enum PartToPrint {
    Header,
    Payload,
}

struct Config {
    token: String,
    part_to_print: PartToPrint,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        let mut part_to_print = PartToPrint::Payload;
        let mut token = String::from("");
        for arg in args {
            match arg.as_str() {
                "--header" => part_to_print = PartToPrint::Header,
                "--payload" => part_to_print = PartToPrint::Payload,
                _ => token = arg.to_string(),
            }
        }
        if token == "" {
            return Err("no token provided");
        }
        return Ok(Config {
            token,
            part_to_print,
        });
    }
}

struct JWTParts {
    header: serde_json::Value,
    payload: serde_json::Value,
}

impl JWTParts {
    fn build(token: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = token.split(".").collect();

        let header_bytes = base64_url::decode(parts[0])?;
        let header: serde_json::Value =
            serde_json::from_str(str::from_utf8(header_bytes.as_slice())?)?;

        let payload_bytes = base64_url::decode(parts[1])?;
        let payload: serde_json::Value =
            serde_json::from_str(str::from_utf8(payload_bytes.as_slice())?)?;
        return Ok(JWTParts { header, payload });
    }
}

fn calculate_output(config: &Config) -> Result<String, Box<dyn Error>> {
    let jwt_parts = JWTParts::build(config.token.as_str());
    match jwt_parts {
        Ok(parts) => match config.part_to_print {
            PartToPrint::Header => return Ok(serde_json::to_string_pretty(&parts.header).unwrap()),
            PartToPrint::Payload => {
                return Ok(serde_json::to_string_pretty(&parts.payload).unwrap())
            }
        },
        Err(_) => return Err("invalid token".into()),
    }
}

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("{}", calculate_output(&config)?);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    if let Err(e) = run(&config) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const token: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    #[test]
    fn it_returns_the_header() {
        let config = Config {
            part_to_print: PartToPrint::Header,
            token: token.to_string(),
        };
        assert_eq!(
            calculate_output(&config).unwrap(),
            "{\n  \"alg\": \"HS256\",\n  \"typ\": \"JWT\"\n}"
        );
    }

    #[test]
    fn it_prints_the_payload() {
        let config = Config {
            part_to_print: PartToPrint::Payload,
            token: token.to_string(),
        };
        assert_eq!(
            calculate_output(&config).unwrap(),
            "{\n  \"iat\": 1516239022,\n  \"name\": \"John Doe\",\n  \"sub\": \"1234567890\"\n}"
        );
    }

    #[test]
    #[should_panic(expected = "invalid token")]
    fn it_prints_error_when_invalid_token() {
        let config = Config {
            part_to_print: PartToPrint::Payload,
            token: "fake-token".to_string(),
        };
        calculate_output(&config).unwrap();
    }
}
