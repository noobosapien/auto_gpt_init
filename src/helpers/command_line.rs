use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stdin, stdout};

#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    AiCall,
    UnitTest,
    Issue,
}

impl PrintCommand {
    pub fn print_agent_msg(&self, agent_pos: &str, agent_statement: &str) {
        let mut stdout: std::io::Stdout = stdout();

        let statement_color: Color = match self {
            Self::AiCall => Color::Cyan,
            Self::UnitTest => Color::Magenta,
            Self::Issue => Color::Red,
        };

        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        print!("Agent: {}:", agent_pos);

        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", agent_statement);

        stdout.execute((ResetColor)).unwrap();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_prints_agent_msg() {
        PrintCommand::AiCall.print_agent_msg("Managing agent", "Testing a process.");
    }
}
