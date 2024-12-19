use std::fs;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    println!("* injecting env vars from secrets...");

    for entry in fs::read_dir("/run/secrets").expect("couldn't read secrets dir") {
        let Ok(entry) = entry else {
            println!("error reading path {}", entry.unwrap_err());
            continue;
        };

        let var_name = entry.file_name().to_string_lossy().to_ascii_uppercase();
        let var_value = fs::read_to_string(entry.path()).expect("couldn't read file");

        println!("     - {}", var_name);
        std::env::set_var(var_name, var_value.trim());
    }

    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let (cmd, args) = args.split_first().expect("no command specified");

    println!(
        "error starting command: {}",
        Command::new(cmd).args(args).exec()
    );
}
