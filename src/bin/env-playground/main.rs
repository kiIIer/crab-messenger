use std::env;
use dotenv::dotenv;

#[macro_use]
extern crate dotenv_codegen;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }

    println!("{}", dotenv!("AUTH0_SERVER_E"));

    Ok(())
}
