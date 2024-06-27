use std::{env, fs, path::Path, str::FromStr};

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

pub static QUESTION_ID: Lazy<String> = Lazy::new(|| {
    env_or_default(
        "QUESTION_ID",
        "0354ea4d-0921-4391-b241-c2f9af72bbfa".to_string(),
    )
});

pub static TEST_PATH: Lazy<String> = Lazy::new(|| {
    env_or_default(
        "TEST_PATH",
        "D:\\Workplace\\CLB F-code\\BE_RODE\\Đề\\BE\\3_KETBAN\\Test".to_string(),
    )
});

#[derive(Debug)]
pub struct TestCase {
    input: String,
    output: String,
    is_visible: bool,
    question_id: Uuid,
}

impl TestCase {
    fn get_testcase_from_dir(
        path: &str,
        quesiton_id: &str,
        extension: bool,
    ) -> Result<Vec<TestCase>, anyhow::Error> {
        let in_dir = Path::new(path).join("in");
        let out_dir = Path::new(path).join("out");
        let mut count: i8 = 0;
        let mut test_cases = Vec::new();
        println!("{:?}", in_dir);

        let in_files = fs::read_dir(in_dir).expect("Cannot open input files!");
        for file in in_files {
            count += 1;
            let mut is_visible: bool = false;
            if count <= 3 {
                is_visible = true;
            }

            let file = file.expect("Failed to read input file");
            let file_path = file.path();
            let file_name = file_path.file_name().expect("Failed to take file name");

            let input_contents =
                fs::read_to_string(&file_path).expect("Failed to read from input file");
            let input_contents = input_contents.trim();

            let out_files_path = if !extension {
                out_dir.join(file_name)
            } else {
                let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                out_dir.join(format!("{}.out", file_name))
            };
            let output_contents =
                fs::read_to_string(out_files_path).expect("Cannot read from output file");
            let output_contents = output_contents.trim();

            test_cases.push(Self {
                input: input_contents.to_string(),
                output: output_contents.to_string(),
                is_visible: is_visible,
                question_id: Uuid::from_str(quesiton_id.trim())?,
            });
        }
        Ok(test_cases)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let test_cases =
        TestCase::get_testcase_from_dir(&TEST_PATH.as_str(), &QUESTION_ID.as_str(), true)
            .expect("Cannot get testcases!");

    println!("{:?}", test_cases);

    let pool = PgPool::connect(&DATABASE_URL).await.unwrap();

    for test_case in test_cases {
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
