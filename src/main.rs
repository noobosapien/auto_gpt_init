mod ai_functions;
mod api;
mod helpers;
mod models;

use helpers::command_line::get_user_response;

fn main() {
    let user_req = get_user_response("What webserver are we building today?");
}
