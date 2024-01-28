use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stdin, stdout};

pub fn get_user_response(questions: &str) -> String {
    let mut stdout: std::io::Stdout = stdout();

    stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
    println!("");
    println!("{}", questions);

    stdout.execute(ResetColor).unwrap();

    let mut user_input: String = String::new();

    stdin()
        .read_line(&mut user_input)
        .expect("Failed to read the response.");

    user_input.trim().to_string()
}
