use std::{env as std_env, sync::LazyLock};

use dotenvy::dotenv;

pub mod env {
    pub const LOAD_BALANCER_ADDRESS_ENV_VAR: &str = "LOAD_BALANCER_ADDRESS";
    pub const WORKERS_ADDRESSES_ENV_VAR: &str = "WORKERS_ADDRESSES";
}

pub const LOAD_BALANCER_NAME: &str = "ULTRA";

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