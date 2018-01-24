extern crate getopts;
extern crate chrono;
use chrono::{DateTime, Utc, FixedOffset, Duration};
use getopts::Options;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::env;


struct Timer {
    name: String,
    start: DateTime<FixedOffset>,
}

fn config_file_path() -> String {
    env::home_dir()
        .expect("Home directory not set")
        .join(".timerconfig")
        .into_os_string()
        .into_string()
        .expect("Path to config failed")
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} COMMAND [NAME]

Commands:
    start: starts a timer with timer name NAME
    stop: stops current timer", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { println!("{}", f.to_string()); return; }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    
    let task = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    if task == "start" {
        let timer_name = if task == "start" && matches.free.len() >= 2 {
            matches.free[1].clone()
        } else {
            print_usage(&program, opts);
            return;
        };

        let timer = get_current_timer();
        
        if let Some(timer) = timer {
            println!("Timer {} currently running", timer.name);
        } else {
            let now = Utc::now().to_rfc2822();
            append(format!("Current: {} {}\n", timer_name, now));
        }
    } else if task == "stop" {
        let timer = get_current_timer();

        if let Some(timer) = timer {
            let duration = Utc::now().signed_duration_since(timer.start);

            delete_line();
            append(format!("{}: {}\n", timer.name, format_duration(duration)));
        } else {
            println!("No current timer");
        }
    }
}

fn append(line: String) {
    let config_file_path = config_file_path();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(config_file_path)
        .unwrap();

    file.write(line.as_bytes()).expect("Could not write to ~/.timerconfig");
}

fn delete_line() {
    let config_file_path = config_file_path();

    Command::new("sed")
        .arg("-i")
        .arg("$d")
        .arg(config_file_path)
        .output()
        .expect("Failed to write to ~/.timerconfig");
}

fn get_current_timer() -> Option<Timer> {
    let config_file_path = config_file_path();
    let curr_timer = Command::new("tail")
        .arg("-n")
        .arg("1")
        .arg(config_file_path)
        .output()
        .expect("Failed to read ~/.timerconfig");

    let curr_timer_string = String::from_utf8_lossy(&curr_timer.stdout);
    let mut curr_timer_fields = curr_timer_string.splitn(3, " ");

    let mut out: Option<Timer> = None;

    if curr_timer_fields.next().expect("No current timer") == "Current:" {
        let name: String = curr_timer_fields.next().expect("Invalid ~/.timerconfig").into();
        let time_str = curr_timer_fields.next().expect("Invalid ~/.timerconfig").trim();
        let datetime = DateTime::parse_from_rfc2822(time_str).expect("Invalid ~/.timerconfig");

        out = Some(Timer{name, start: datetime});
    }

    out
}

fn format_duration(duration: Duration) -> String {
    let nums = [
        duration.num_weeks(),
        duration.num_days() % 7,
        duration.num_hours() % 24,
        duration.num_minutes() % 60,
        duration.num_seconds() % 60,
    ];
    let names = ["weeks", "days", "hours", "minutes", "seconds"];
    let mut out = String::new();
    let mut found = false;
    
    for i in 0..5 {
        if found || nums[i] != 0 {
            found = true;
            out = format!("{} {} {}", out, nums[i], names[i]);
        }
    }

    out
}

