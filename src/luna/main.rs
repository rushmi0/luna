use std::io::{self, Write};
use std::sync::Arc;

use luna::{LocalValue, LuaOption, LuaStdLib, LuaVersion, LunaVM, Vm};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Command {
    Help,
    Version,
    RunFile(String),
    Repl,
}

fn main() {
    match parse_args() {
        Command::Help => print_help(),
        Command::Version => print_version(&start_vm()),
        Command::RunFile(path) => run_file(&start_vm(), &path),
        Command::Repl => repl(&start_vm()),
    }
}

fn parse_args() -> Command {
    match std::env::args().nth(1).as_deref() {
        Some("-h") | Some("--help") => Command::Help,
        Some("-v") | Some("--version") => Command::Version,
        Some("--repl") => Command::Repl,
        Some(path) => Command::RunFile(path.to_string()),
        None => Command::Repl,
    }
}

fn print_help() {
    println!(
        "{NAME} {VERSION} — a Lua 5.4 runtime

USAGE:
    luna [OPTIONS] [SCRIPT]

ARGS:
    <SCRIPT>    Path to a Lua script to run. If omitted, starts an interactive REPL.

OPTIONS:
    -h, --help       Print this help message and exit
    -v, --version    Print version information and exit
        --repl       Start the REPL explicitly (default when SCRIPT is omitted)

REPL:
    .exit            Quit the REPL
    Ctrl+D           Quit the REPL (EOF)"
    );
}

fn print_version(vm: &Vm) {
    println!("{NAME} {VERSION} ({})", vm.version());
    println!(
        "Luna Runtime Environment (features: {})",
        active_features().join(", ")
    );
    println!(
        "Luna VM ({}-{}, single-threaded async runtime)",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
}

fn active_features() -> Vec<&'static str> {
    let mut features = Vec::new();
    if cfg!(feature = "send") {
        features.push("send");
    }
    if cfg!(feature = "async") {
        features.push("async");
    }
    if cfg!(feature = "serialize") {
        features.push("serialize");
    }
    features
}

fn start_vm() -> Arc<Vm> {
    LunaVM::new(LuaOption {
        version: LuaVersion::Lua54,
        stdlib: LuaStdLib::All,
        memory_limit: None,
        instruction_limit: None,
        timeout: None,
    })
    .start()
    .unwrap_or_else(|e| {
        eprintln!("luna: failed to start VM: {e}");
        std::process::exit(1);
    })
}

fn run_file(vm: &Vm, path: &str) {
    if let Err(e) = vm.run_file(path.to_string()) {
        eprintln!("luna: {e}");
        std::process::exit(1);
    }
}

fn repl(vm: &Vm) {
    println!(
        "{NAME} {VERSION} ({}) — .help for options, .exit to quit",
        vm.version()
    );

    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        print!("> ");
        io::stdout().flush().ok();

        line.clear();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            println!();
            break; // Ctrl+D
        }

        let source = line.trim();
        match source {
            "" => continue,
            ".exit" => break,
            ".help" => {
                println!(".exit            Quit the REPL");
                println!("Ctrl+D           Quit the REPL (EOF)");
                continue;
            }
            _ => {}
        }

        match vm.run_named(source.to_string(), "stdin".to_string()) {
            Ok(value) => println!("{}", format_value(&value)),
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn format_value(value: &LocalValue) -> String {
    match value {
        LocalValue::Nil => "nil".to_string(),
        LocalValue::Boolean(b) => b.to_string(),
        LocalValue::Integer(i) => i.to_string(),
        LocalValue::Number(n) => n.to_string(),
        LocalValue::LuaString(s) => format!("{s:?}"),
    }
}
