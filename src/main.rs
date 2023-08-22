use std::process::Command;
use std::time::Duration;
use chrono::NaiveTime;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args_ref:Vec<&str> = args.iter().map(String::as_str).collect();
    let result = try_parse(&args_ref, chrono::Local::now().time());
    match result {
        Err(e) => println!("Error: {}", e),
        Ok(reminder) => {
            println!("setting up {:?}", reminder);
            std::thread::sleep(reminder.time);
            let _output = Command::new("osascript")
                .arg("-e")
                // .arg(format!("display notification \"{}\" with title \"Reminder\"", reminder.action))
                .arg(format!("display alert \"{}\"", reminder.action))
                .output()
                .expect("command failed");

            // println!("status: {}", _output.status);
            // println!("stdout: {}", String::from_utf8_lossy(&_output.stdout));
            // println!("stderr: {}", String::from_utf8_lossy(&_output.stderr));
        },
    }
}
const ACTION_MARKERS: [&str; 2] = ["to", "that"];

#[derive(Debug, PartialEq)]
struct Reminder {
    time: Duration,
    action: String,
}

fn try_parse(words: &[&str], now: NaiveTime) -> Result<Reminder, String> {
    let mut i = 1;
    if words.len() <= i {
        return Err("Expected an action, 'in <time interval>', or 'me'".to_string());
    }
    if words[i] == "me" {
        i += 1;
    }
    if words.len() <= i {
        return Err("Expected an action or 'in <time interval>'".to_string());
    }
    if words[i] == "in" {
        let time = parse_time_diff(words, &mut i)?;
        if words.len() <= i {
            return Err("Expected an action".to_string());
        }
        if ACTION_MARKERS.contains(&words[i]) {
            i += 1;
        }
        let action = words[i..].join(" ");
        Ok(Reminder {
            time,
            action,
        })
    } else if words[i] == "at" {
        let time = parse_time(words, &mut i, now)?;
        if words.len() <= i {
            return Err("Expected an action".to_string());
        }
        if ACTION_MARKERS.contains(&words[i]) {
            i += 1;
        }
        let action = words[i..].join(" ");
        Ok(Reminder {
            time,
            action,
        })
    } else {
        if ACTION_MARKERS.contains(&words[i]) {
            i += 1;
        }
        if words.len() <= i {
            return Err("Expected an action".to_string());
        }
        let time_diff_index_opt = words.iter().enumerate().rfind(|w| w.1 == &"in").map_or(None, |w| Some(w.0));
        if let Some(mut time_index) = &time_diff_index_opt {
            let action = words[i..time_index].join(" ");
            let time = parse_time_diff(&words, &mut time_index)?;
            return Ok(Reminder {
                time,
                action,
            });
        }

        let time_index_opt = words.iter().enumerate().rfind(|w| w.1 == &"at").map_or(None, |w| Some(w.0));
        if let Some(mut time_index) = &time_index_opt {
            let action = words[i..time_index].join(" ");
            let time = parse_time(&words, &mut time_index, now)?;
            return Ok(Reminder {
                time,
                action,
            });
        }
        let action = words[i..].join(" ");
        return Ok(Reminder {
            time: Duration::from_secs(0),
            action,
        });
    }
}

fn parse_time_diff(words: &[&str], i: &mut usize) -> Result<Duration, String> {
    if words.len() <= *i {
        return Err("Expected a time interval".to_string());
    }
    if words[*i] == "in" {
        *i += 1;
    } else {
        return Ok(Duration::from_secs(0));
    }
    if words.len() <= *i {
        return Err("Expected a time interval".to_string());
    }
    let time = match words[*i].parse::<u64>() {
        Ok(t) => t,
        Err(_) => return Err("Expected a time interval".to_string()),
    };
    *i += 1;
    if words.len() <= *i {
        return Err("Expected a time unit".to_string());
    }
    let time = match words[*i] {
        "second" | "seconds" => Duration::from_secs(time),
        "minute" | "minutes" => Duration::from_secs(time * 60),
        "hour" | "hours" => Duration::from_secs(time * 60 * 60),
        _ => return Err("Expected a time unit".to_string()),
    };
    *i += 1;
    Ok(time)
}

fn parse_time(words: &[&str], i: &mut usize, now: NaiveTime) -> Result<Duration, String> {
    if words.len() <= *i {
        return Err("Expected a time".to_string());
    }
    if words[*i] == "at" {
        *i += 1;
    } else {
        return Ok(Duration::from_secs(0));
    }
    if words.len() <= *i {
        return Err("Expected a time".to_string());
    }
    let time = match NaiveTime::parse_from_str(words[*i], "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            match NaiveTime::parse_from_str(words[*i], "%H:%M:%S") {
                Ok(t) => t,
                Err(_) => return Err("Expected a time".to_string()),
            }
        },
    };
    *i += 1;
    let time = time.signed_duration_since(now);
    if time.num_seconds() < 0 {
        return Err("Time should not be in the past".to_string());
    }
    Ok(Duration::from_secs(time.num_seconds() as u64))
}


#[cfg(test)]
mod tests {
    use chrono::NaiveTime;
    use super::*;

    fn now() -> NaiveTime {
        return NaiveTime::from_hms_opt(13, 51, 30).unwrap();
    }

    #[test]
    fn test_parse_empty() {
        let result = try_parse(&[], now());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_only_command() {
        let result = try_parse(&["remind"], now());
        assert!(result.is_err());
    }

    fn to_slice(s: &str) -> Vec<&str> {
        s.split_ascii_whitespace().collect::<Vec<&str>>()
    }

    #[test]
    fn test_parse_only_me() {
        let result = try_parse(&to_slice("remind me"), now());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_only_action() {
        let result = try_parse(&to_slice("remind work"), now());
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(0),
            action: "work".to_string(),
        }));
    }

    #[test]
    fn test_parse_time_and_action() {
        let result = try_parse(&to_slice("remind in 5 minutes to ask"), now());
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(5 * 60),
            action: "ask".to_string(),
        }));
    }

    #[test]
    fn test_parse_to_action_and_time() {
        let result = try_parse(&to_slice("remind to ask in 5 minutes"), now());
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(5 * 60),
            action: "ask".to_string(),
        }));
    }

    #[test]
    fn test_parse_me_and_action_and_time() {
        let result = try_parse(&to_slice("remind me to ask something in 5 minutes"), now());
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(5 * 60),
            action: "ask something".to_string(),
        }));
    }

    #[test]
    fn test_parse_action_and_time() {
        let result = try_parse(&to_slice("remind closing something in 5 minutes"), now());
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(5 * 60),
            action: "closing something".to_string(),
        }));
    }

    #[test]
    fn test_parse_action_and_at_time() {
        let now = NaiveTime::from_hms_opt(13, 51, 30).unwrap();
        let result = try_parse(&to_slice("remind me to do something at 14:10"), now);
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(18 * 60 + 30),
            action: "do something".to_string(),
        }));
    }

    #[test]
    fn test_parse_action_and_at_time_seconds() {
        let now = NaiveTime::from_hms_opt(13, 51, 30).unwrap();
        let result = try_parse(&to_slice("remind me to do something at 14:10:31"), now);
        assert_eq!(result, Ok(Reminder {
            time: Duration::from_secs(19 * 60 + 1),
            action: "do something".to_string(),
        }));
    }

    #[test]
    fn test_parse_negative_time() {
        let now = NaiveTime::from_hms_opt(13, 51, 30).unwrap();
        let result = try_parse(&to_slice("remind me to do something at 7:10:31"), now);
        assert!(result.is_err());
    }
}
