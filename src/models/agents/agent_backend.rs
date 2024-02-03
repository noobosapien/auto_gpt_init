use crate::ai_functions::aifunc_backend::{
    print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
    print_rest_api_endpoints,
};
use crate::helpers::general::{
    check_status_code, read_code_template_contents, read_executable_main_contents,
    save_api_endpoints, save_backend_code, WEB_SERVER_PROJECT_PATH,
};

use crate::helpers::command_line::{confirm_safe_code, PrintCommand};
use crate::helpers::general::ai_task_request;
use crate::models::agents_basic::basic_agents::{AgentState, BasicAgent};

use crate::models::agents::agent_traits::{FactSheet, RouteObject, SpecialFunctions};

use async_trait::async_trait;
use core::panic;
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

                    PrintCommand::UnitTest.print_agent_msg(
                        &self.attributes.position.as_str(),
                        "Backend code unit testing: building the project",
                    );

                    let build_backend_server: std::process::Output = Command::new("cargo")
                        .arg("build")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .expect("Failed to build the backend application.");

                    if build_backend_server.status.success() {
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_msg(
                            &self.attributes.position.as_str(),
                            "Backend code unit testing: Test server build succesful.",
                        );
                    } else {
                        let error_arr: Vec<u8> = build_backend_server.stderr;
                        let error_str: String = String::from_utf8(error_arr).unwrap();

                        self.bug_count += 1;
                        self.bug_errors = Some(error_str);

                        if self.bug_count > 2 {
                            PrintCommand::Issue.print_agent_msg(
                                &self.attributes.position.as_str(),
                                "Too many bugs found in code. Exiting",
                            );

                            panic!("Error: too many bugs.");
                        }

                        self.attributes.state = AgentState::Working;
                        continue;
                    }

                    let api_endpoints_str: String = self.call_extract_api_endpoints().await;
                    let api_endpoints: Vec<RouteObject> =
                        serde_json::from_str(&api_endpoints_str.as_str())
                            .expect("Failed to create the API endpoints.");

                    let check_endpoints: Vec<RouteObject> = api_endpoints
                        .iter()
                        .filter(|&route_object| {
                            route_object.method == "get" && route_object.is_route_dynamic == "false"
                        })
                        .cloned()
                        .collect();

                    factsheet.api_endpoint_schema = Some(check_endpoints.clone());

                    PrintCommand::UnitTest
                        .print_agent_msg(&self.attributes.position.as_str(), "Starting web server");

                    let mut run_backend_server: std::process::Child = Command::new("cargo")
                        .arg("run")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run the backend application.");

                    PrintCommand::UnitTest.print_agent_msg(
                        &self.attributes.position.as_str(),
                        "Launching tests on the server",
                    );

                    let seconds_sleep: Duration = Duration::from_secs(5);
                    time::sleep(seconds_sleep).await;

                    for endpoint in check_endpoints {
                        let testing_msg: String = format!("Testing endpoint: {}", endpoint.route);
                        PrintCommand::UnitTest.print_agent_msg(
                            &self.attributes.position.as_str(),
                            &testing_msg.as_str(),
                        );

                        let url: String = format!("http://localhost:8080{}", endpoint.route);

                        let client = Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()
                            .unwrap();

                        match check_status_code(&client, &url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    let err_msg: String =
                                        format!("Failed to call the endpoint: {}", endpoint.route);

                                    PrintCommand::Issue.print_agent_msg(
                                        &self.attributes.position.as_str(),
                                        &err_msg.as_str(),
                                    );
                                }
                            }
                            Err(e) => {
                                run_backend_server
                                    .kill()
                                    .expect("Failed to kill backend server.");

                                let err_msg: String = format!("Failed to kill the server.");

                                PrintCommand::Issue.print_agent_msg(
                                    &self.attributes.position.as_str(),
                                    &err_msg.as_str(),
                                );
                            }
                        }
                    }

                    save_api_endpoints(&api_endpoints_str);
                    PrintCommand::UnitTest.print_agent_msg(
                        &self.attributes.position.as_str(),
                        "Backend testing is completed.",
                    );

                    run_backend_server
                        .kill()
                        .expect("Failed to kill the backend server.");

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
            "project_description": "build a website that returns current time.",
            "project_scope":  {
                    "is_crud_required": false,
                    "is_user_login_and_logout": false,
                    "is_external_urls_required": false
                },
            
            "external_urls": [],
            
            "backend_code": null,
            "api_endpoint_schema": null
        }
        "#;

        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();

        agent.attributes.state = AgentState::UnitTesting;

        agent
            .execute(&mut factsheet)
            .await
            .expect("Failed to execute Backend Developer");
    }
}
