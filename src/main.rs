extern crate dotenv;
extern crate jadwiga;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use jadwiga::config::RawConfig;

fn main() {
    dotenv::dotenv()
        .expect("Failed to run dotenv");
    
    let get_env = |var: &str| env::var(var)
        .expect(&format!("Failed to get environment variable {}", var));
    
    let raw_config = RawConfig {
        db_url: get_env("JADWIGA_DATABASE_URL"),
        pub_key: {
            let pub_key_path = get_env("JADWIGA_PUBLIC_KEY");
            let mut pub_key_file = File::open(pub_key_path)
                .expect("Failed to open public key file");

            let mut pub_key = String::new();
            pub_key_file.read_to_string(&mut pub_key)
                .expect("Failed to read public key file");

            pub_key
        },
        host: get_env("JADWIGA_HOST"),
        root_url: get_env("JADWIGA_ROOT_URL"),
        actor_username: get_env("JADWIGA_USERNAME"),
        actor_name: get_env("JADWIGA_NAME"),
        media_dir: get_env("JADWIGA_MEDIA_DIR")
    };

    let config = raw_config.validate()
        .expect("Failed to validate config");
    
    jadwiga::run(config)
        .expect("Failed to run jadwiga");
}

