use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = try_parse(&args.iter().map(String::as_str).collect::<Vec<_>>());
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
    time: std::time::Duration,
    action: String,
}

fn try_parse(words: &[&str]) -> Result<Reminder, String> {
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
        let time = parse_time(words, &mut i)?;
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
        let time_index_opt = words.iter().enumerate().rfind(|w| w.1 == &"in").map_or(None, |w| Some(w.0));
        return match time_index_opt {
            Some(mut time_index) => {
                let action = words[i..time_index].join(" ");
                let time = parse_time(&words, &mut time_index)?;
                Ok(Reminder {
                    time,
                    action,
                })
            },
            None => {
                let action = words[i..].join(" ");
                Ok(Reminder {
                    time: std::time::Duration::from_secs(0),
                    action,
                })
            }
        };
    }
}

fn parse_time(words: &[&str], i: &mut usize) -> Result<std::time::Duration, String> {
    if words.len() <= *i {
        return Err("Expected a time interval".to_string());
    }
    if words[*i] == "in" {
        *i += 1;
    } else {
        return Ok(std::time::Duration::from_secs(0));
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
        "second" | "seconds" => std::time::Duration::from_secs(time),
        "minute" | "minutes" => std::time::Duration::from_secs(time * 60),
        "hour" | "hours" => std::time::Duration::from_secs(time * 60 * 60),
        _ => return Err("Expected a time unit".to_string()),
    };
    *i += 1;
    Ok(time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = try_parse(&[]);
        assert_eq!(result.is_ok(), false);
    }
    #[test]
    fn test_parse_only_command() {
        let result = try_parse(&["remind"]);
        assert_eq!(result.is_ok(), false);
    }

    fn to_slice(s: &str) -> Vec<&str> {
        s.split_ascii_whitespace().collect::<Vec<&str>>()
    }
    #[test]
    fn test_parse_only_me() {
        let result = try_parse(&to_slice("remind me"));
        assert_eq!(result.is_ok(), false);
    }
    #[test]
    fn test_parse_only_action() {
        let result = try_parse(&to_slice("remind work"));
        assert_eq!(result, Ok(Reminder {
            time: std::time::Duration::from_secs(0),
            action: "work".to_string(),
        }));
    }
    #[test]
    fn test_parse_time_and_action() {
        let result = try_parse(&to_slice("remind in 5 minutes to ask"));
        assert_eq!(result, Ok(Reminder {
            time: std::time::Duration::from_secs(5 * 60),
            action: "ask".to_string(),
        }));
    }
    #[test]
    fn test_parse_to_action_and_time() {
        let result = try_parse(&to_slice("remind to ask in 5 minutes"));
        assert_eq!(result, Ok(Reminder {
            time: std::time::Duration::from_secs(5 * 60),
            action: "ask".to_string(),
        }));
    }
    #[test]
    fn test_parse_me_and_action_and_time() {
        let result = try_parse(&to_slice("remind me to ask something in 5 minutes"));
        assert_eq!(result, Ok(Reminder {
            time: std::time::Duration::from_secs(5 * 60),
            action: "ask something".to_string(),
        }));
    }
    #[test]
    fn test_parse_action_and_time() {
        let result = try_parse(&to_slice("remind closing something in 5 minutes"));
        assert_eq!(result, Ok(Reminder {
            time: std::time::Duration::from_secs(5 * 60),
            action: "closing something".to_string(),
        }));
    }
}