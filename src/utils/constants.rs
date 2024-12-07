use std::{env as std_env, sync::LazyLock};

use dotenvy::dotenv;

pub const LOAD_BALANCER_NAME: &str = "ultra";
pub const DB_ADDRESS: &str = "127.0.0.1:5432";
pub const MAX_DB_CONNECTIONS: u32 = 100;
pub const TABLE_NAME: &str = "workers"; // postgres table name
pub const DF_TABLE_NAME: &str = "workers"; // datafusion table name

pub mod env {
    pub const LOAD_BALANCER_ADDRESS_ENV_VAR: &str = "LOAD_BALANCER_ADDRESS";
    pub const WORKERS_ADDRESSES_ENV_VAR: &str = "WORKERS_ADDRESSES";
    pub const DB_NAME_ENV_VAR: &str = "DB_NAME";
    pub const DB_USER_ENV_VAR: &str = "DB_USER";
    pub const PASSWORD_ENV_VAR: &str = "DB_PASSWORD";
}

pub static LOAD_BALANCER_ADDRESS_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::LOAD_BALANCER_ADDRESS_ENV_VAR)
        .expect("LOAD_BALANCER_ADDRESS must be set.");
    if secret.is_empty() {
        panic!("LOAD_BALANCER_ADDRESS must not be empty.");
    }
    secret
});

pub static WORKERS_ADDRESSES_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::WORKERS_ADDRESSES_ENV_VAR)
        .expect("WORKERS_ADDRESSES must be set.");
    if secret.is_empty() {
        panic!("WORKERS_ADDRESSES must not be empty.");
    }
    secret
});

pub static DB_NAME_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::DB_NAME_ENV_VAR)
        .expect("DB_NAME_SECRET must be set.");
    if secret.is_empty() {
        panic!("DB_NAME_SECRET must not be empty.");
    }
    secret
});

pub static DB_USER_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::DB_USER_ENV_VAR)
        .expect("DB_USER_NAME_SECRET must be set.");
    if secret.is_empty() {
        panic!("DB_USER_NAME_SECRET must not be empty.");
    }
    secret
});

pub static DB_PASSWORD_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std_env::var(env::PASSWORD_ENV_VAR)
        .expect("PASSWORD_SECRET must be set.");
    if secret.is_empty() {
        panic!("PASSWORD_SECRET must not be empty.");
    }
    secret
});