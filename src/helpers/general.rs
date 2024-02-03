use super::command_line::PrintCommand;
use crate::{api::call_request::call_gpt, models::general::llm::Message};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::fs;

pub const CODE_TEMPlATE_PATH: &str = "/home/migara/Desktop/projects/actix_template/src/template.rs";
pub const EXEC_MAIN_PATH: &str = "/home/migara/Desktop/projects/actix_template/src/main.rs";
pub const API_SCHEMA_PATH: &str =
    "/home/migara/Desktop/projects/auto_gpt_init/schemas/api_schema.json";
pub const WEB_SERVER_PROJECT_PATH: &str = "/home/migara/Desktop/projects/actix_template/";

pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, func_input: &str) -> Message {
    let ai_function_str = ai_func(func_input);
    dbg!(ai_function_str);

    let msg: String = format!(
        "FUNCTION: {}
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentary. Here is the input of the function: {}.
    Print out what the function will return.",
        ai_function_str, func_input
    );

    Message {
        role: "system".to_string(),
        content: msg,
    }
}

pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let extended_msg: Message = extend_ai_function(function_pass, &msg_context);

    PrintCommand::AiCall.print_agent_msg(agent_position, agent_operation);

    let llm_response: Result<String, Box<dyn std::error::Error + Send>> =
        call_gpt(vec![extended_msg.clone()]).await;

    match llm_response {
        Ok(llm_resp) => llm_resp,
        Err(_) => call_gpt(vec![extended_msg.clone()])
            .await
            .expect("Failed to call chatGPT"),
    }
}

pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response: String =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;

    let decoded_response: T =
        serde_json::from_str(llm_response.as_str()).expect("Failed to decode the AI response.");

    decoded_response
}

pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response: reqwest::Response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}

pub fn read_code_template_contents() -> String {
    let path: String = CODE_TEMPlATE_PATH.to_string();
    fs::read_to_string(path).expect("Couldn't read the file.")
}

pub fn read_executable_main_contents() -> String {
    let path: String = EXEC_MAIN_PATH.to_string();
    fs::read_to_string(path).expect("Couldn't read the file.")
}

pub fn save_backend_code(contents: &String) {
    let path: String = EXEC_MAIN_PATH.to_string();
    fs::write(path, contents).expect("Couldn't write to file.");
}

pub fn save_api_endpoints(api_endpoints: &String) {
    let path: String = API_SCHEMA_PATH.to_string();
    fs::write(path, api_endpoints).expect("Couldn't write to api endpoints file.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::aifunc_managing::convert_user_input_to_goal;

    #[test]
    fn tests_extending_function() {
        let res = extend_ai_function(convert_user_input_to_goal, "dummy variable");
        assert_eq!(res.role, "system".to_string());
    }

    #[tokio::test]
    async fn tests_ai_task_request() {
        let ai_func_param =
            "Build me a webserver with endpoints of the current ethereum price.".to_string();

        let res = ai_task_request(
            ai_func_param,
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        )
        .await;

        dbg!(&res);
        assert!(res.len() > 20);
    }
}
