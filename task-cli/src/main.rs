use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    // The command to execute
    command: String,
}
fn main() {
    let args = Cli::parse();

    println!("{:?}", args);
}
