use std::{env, fs, str::FromStr};

use anyhow::Context;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

pub fn env_or_default<T: FromStr>(env_name: &'static str, default: T) -> T {
    match env::var(env_name) {
        Err(_) => default,
        Ok(raw) => match raw.parse() {
            Ok(value) => value,
            Err(_) => default,
        },
    }
}

pub static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    env_or_default(
        "DATABASE_URL",
        "postgres://user:password@host/database".to_string(),
    )
});

#[derive(Debug)]
struct TestCase {
    input: String,
    output: String,
    is_visible: bool,
    question_id: Uuid,
}
impl FromStr for TestCase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .split_once("input:")
            .context("Failed to remove first line")?
            .1;

        let (input, s) = s.split_once("output:").context("Failed to get input")?;
        let (output, s) = s.split_once("isVisible:").context("Failed to get output")?;
        let (is_visible, question_id) = s
            .split_once("question_id:")
            .context("Failed to get is_visible")?;

        let input = input.trim().to_string();
        let output = output.trim().to_string();
        let is_visible = match is_visible.trim() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        };
        let question_id = Uuid::from_str(question_id.trim())?;

        Ok(TestCase {
            input,
            output,
            is_visible,
            question_id,
        })
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let pool = PgPool::connect(&DATABASE_URL).await.unwrap();

    let args: Vec<String> = env::args().collect();
    let test_case_dir_name = &args[1];
    for test_case_file in fs::read_dir(test_case_dir_name).unwrap() {
        let test_case_file_name = test_case_file.unwrap().path();
        let test_case_raw = fs::read_to_string(test_case_file_name).unwrap();
        let test_case = TestCase::from_str(&test_case_raw).unwrap();
        eprintln!("DEBUGPRINT[1]: main.rs:81: test_case={:#?}", test_case);

        sqlx::query!(
            r#"
INSERT INTO test_cases(question_id, input, output, is_visible)
VALUES ($1, $2, $3, $4)
        "#,
            test_case.question_id,
            test_case.input,
            test_case.output,
            test_case.is_visible
        )
        .execute(&pool)
        .await
        .unwrap();
    }
}
