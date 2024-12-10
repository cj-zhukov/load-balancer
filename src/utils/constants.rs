use std::{env as std_env, sync::LazyLock};

use dotenvy::dotenv;
use secrecy::Secret;

pub const LOAD_BALANCER_NAME: &str = "ultra";
pub const PG_MAX_DB_CONS: u32 = 100;
pub const PG_TABLE_NAME: &str = "workers"; // postgres table name
pub const DF_TABLE_NAME: &str = "workers"; // datafusion table name

pub mod env {
    pub const PG_DATABASE_URL_ENV_VAR: &str = "PG_DATABASE_URL";
    pub const LOAD_BALANCER_ADDRESS_ENV_VAR: &str = "LOAD_BALANCER_ADDRESS";
    pub const WORKERS_ADDRESSES_ENV_VAR: &str = "WORKERS_ADDRESSES";
}

pub static PG_DATABASE_URL: LazyLock<Secret<String>> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::PG_DATABASE_URL_ENV_VAR)
        .expect("DATABASE_URL_ENV_VAR must be set.");
    if secret.is_empty() {
        panic!("DATABASE_URL_ENV_VAR must not be empty.");
    }
    Secret::new(secret)
});

pub static LOAD_BALANCER_ADDRESS_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::LOAD_BALANCER_ADDRESS_ENV_VAR)
        .expect("LOAD_BALANCER_ADDRESS must be set.");
    if secret.is_empty() {
        panic!("LOAD_BALANCER_ADDRESS must not be empty.");
    }
    secret
});

// pub static WORKERS_ADDRESSES_SECRET: LazyLock<String> = LazyLock::new(|| {
//     dotenv().ok();
//     let secret = std_env::var(env::WORKERS_ADDRESSES_ENV_VAR)
//         .expect("WORKERS_ADDRESSES must be set.");
//     if secret.is_empty() {
//         panic!("WORKERS_ADDRESSES must not be empty.");
//     }
//     secret
// });