extern crate dotenv;
extern crate jadwiga;

use std::env;

use jadwiga::config::RawConfig;

fn main() {
    dotenv::dotenv()
        .expect("Failed to run dotenv");

    let get_env = |var: &str| env::var(var)
        .expect(&format!("Failed to get environment variable {}", var));
    
    let raw_config = RawConfig {
        db_url: get_env("DATABASE_URL"),
        root_url: get_env("ROOT_URL"),
        actor_username: get_env("ACTOR_USERNAME"),
        actor_name: get_env("ACTOR_NAME")
    };

    let config = raw_config.validate()
        .expect("Failed to validate config");
    
    jadwiga::run(config)
        .expect("Failed to run jadwiga");
}

