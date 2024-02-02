use crate::ai_functions::aifunc_backend::{
    print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
    print_rest_api_endpoints,
};
use crate::helpers::general::{
    check_status_code, read_code_template_contents, read_executable_main_contents,
    save_api_endpoints, save_backend_code,
};

use crate::helpers::command_line::{confirm_safe_code, PrintCommand};
use crate::helpers::general::ai_task_request;
use crate::models::agents_basic::basic_agents::{AgentState, BasicAgent};

use crate::models::agents::agent_traits::{FactSheet, RouteObject, SpecialFunctions};

use async_trait::async_trait;
use crossterm::style::Color;
use crossterm::style::{ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;
use reqwest::Client;
use std::fs;
use std::io::{stdin, stdout};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]

pub struct AgentBackendDeveloper {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDeveloper {
    pub fn new() -> Self {
        let attributes = BasicAgent {
            objective: "Develops code for backend server and the json database".to_string(),
            position: "Backend developer".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        Self {
            attributes,
            bug_errors: None,
            bug_count: 0,
        }
    }

    async fn call_initial_backend_code(&mut self, factsheet: &mut FactSheet) {
        let code_template_str: String = read_code_template_contents();

        let msg_context: String = format!(
            "CODE TEMPLATE: {} \n PROJECT_DESCRIPTION: {} \n",
            code_template_str, factsheet.project_description
        );

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_improved_backend_code(&mut self, factsheet: &mut FactSheet) {
        let code_template_str: String = read_code_template_contents();

        let msg_context: String = format!(
            "CODE TEMPLATE: {:?} \n PROJECT_DESCRIPTION: {:?} \n",
            factsheet.backend_code, factsheet
        );

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_fix_code_bugs(&mut self, factsheet: &mut FactSheet) {
        let code_template_str: String = read_code_template_contents();

        let msg_context: String = format!(
            "BROKEN_CODE: {:?} \n ERROR_BUGS: {:?} \n
            THIS FUNCTION ONLY OUTPUTS THE CODE. JUST THE WORKING CODE NOTHING MORE",
            factsheet.backend_code, self.bug_errors
        );

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_extract_api_endpoints(&self) -> String {
        let backend_code = read_executable_main_contents();

        let msg_context: String = format!("CODE_INPUT: {}", backend_code);

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;

        ai_response
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDeveloper {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.attributes.state != AgentState::Finishing {
            match &self.attributes.state {
                AgentState::Discovery => {
                    self.call_initial_backend_code(factsheet).await;
                    self.attributes.state = AgentState::Working;
                    continue;
                }
                AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improved_backend_code(factsheet).await;
                    } else {
                        self.call_fix_code_bugs(factsheet).await;
                    }

                    self.attributes.state = AgentState::UnitTesting;
                    continue;
                }
                AgentState::UnitTesting => {
                    PrintCommand::UnitTest.print_agent_msg(
                        &self.attributes.position.as_str(),
                        "Backend code unit testing: ensuring safe code",
                    );

                    let is_safe_code: bool = confirm_safe_code();

                    if !is_safe_code {
                        panic!("Stopped mid way.");
                    }

                    self.attributes.state = AgentState::Finishing;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_backend_agent() {
        let mut agent = AgentBackendDeveloper::new();

        let factsheet_str = r#"
        {
            "project_description": "build a website that fetches and tracks fitnedd progress with timezone information.",
            "project_scope":  {
                    "is_crud_required": true,
                    "is_user_login_and_logout": true,
                    "is_external_urls_required": true
                },
            
            "external_urls": [
                    "http://worldtimeapi.org/api/timezone"
                ],
            
            "backend_code": null,
            "api_endpoint_schema": null
        }
        "#;

        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();

        agent
            .execute(&mut factsheet)
            .await
            .expect("Failed to execute Backend Developer");
    }
}
