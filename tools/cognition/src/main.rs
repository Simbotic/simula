use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::{
    Write, {BufReader, Read},
};
use tokio::runtime::Runtime;

// Define the YAML dialog object
#[derive(Serialize, Deserialize)]
struct Dialog {
    id: String,
    text: String,
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    choice: String,
    next_id: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: usize,
    model: String,
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIChoice {
    text: String,
    index: usize,
    logprobs: Option<OpenAILogprobs>,
    finish_reason: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAILogprobs {
    top_logprobs: HashMap<String, f64>,
    text_offset: Vec<usize>,
}

// Define the YAML prompt_decision template object
struct DecisionPromptTemplate(String);

impl DecisionPromptTemplate {
    fn new(file_path: &str) -> Self {
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        Self(contents)
    }

    fn format(&self, decision_prompt: &str, choices: &str, user_response: &str) -> String {
        self.0
            .replace("{{ decision_prompt }}", decision_prompt)
            .replace("{{ choices }}", choices)
            .replace("{{ user_response }}", user_response)
    }
}

async fn send_request(prompt: &str) -> Result<OpenAIResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!(
            "Bearer {}",
            std::env::var("OPENAI_API_KEY").unwrap()
        ))?,
    );
    let response = client
        .post("https://api.openai.com/v1/completions")
        .headers(headers)
        .json(&json!({
            "model": "text-davinci-003",
            "prompt": prompt,
            "suffix": "\n\n",
            "temperature": 0.7,
            "max_tokens": 100,
            "top_p": 1,
            "frequency_penalty": 0,
            "presence_penalty": 0,
        }))
        .send()
        .await?;
    let response_text = response.text().await?;
    let response_json: OpenAIResponse = serde_json::from_str(&response_text)?;
    Ok(response_json)
}

async fn run_dialog() -> Result<(), Box<dyn std::error::Error>> {
    // Load the YAML file
    let file = File::open("tools/cognition/dialog.yaml")?;
    let reader = BufReader::new(file);
    let dialog: Vec<Dialog> = serde_yaml::from_reader(reader)?;

    // Load the YAML prompt_decision template file
    let decision_prompt_template =
        DecisionPromptTemplate::new("tools/cognition/decision_prompt_template.yaml");

    let agent = "Agent";

    // Initialize the dialog
    let mut current_id = "greeting".to_string();
    let mut predicting_choice = false;
    let mut user_response = String::new();

    loop {
        // Find the current dialog object
        let current_dialog = dialog
            .iter()
            .find(|obj| obj.id == current_id)
            .ok_or("Oops, something went wrong. Please try again.")?;

        // Print the current text and choices
        println!("{}: {}", agent, current_dialog.text);
        for choice in &current_dialog.choices {
            println!("- {}", choice.choice);
        }

        // Map choices to choices.choice
        let choices: Vec<String> = current_dialog
            .choices
            .iter()
            .map(|choice| choice.choice.trim().to_string())
            .collect();
        let choices = choices.join("\n - ");

        // Prompt the user for input, unless we are predicting the choice
        if !predicting_choice {
            let mut user_input = String::new();
            print!("User: ");
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut user_input)?;
            user_response = user_input.trim().to_string();
        }

        // Create the prompt_decision
        let decision_prompt = current_dialog.text.clone();
        let decision_prompt =
            decision_prompt_template.format(&decision_prompt, &choices, &user_response);
        println!("++++++ PROMPT ++++++");
        print!("{}", decision_prompt);

        // Send the request to OpenAI asynchronously
        let response_json = send_request(&decision_prompt).await?;
        // println!("RESPONSE: {:?}", serde_json::to_string(&response_json));

        // Get the first choice from the response
        let choice = response_json
            .choices
            .get(0)
            .ok_or("OpenAI did not return any choices")?
            .text
            .trim()
            .to_string();

        println!("{}", choice);
        println!("--------------------");

        // Try to match the user's response with one of the choices
        let choice_index = current_dialog
            .choices
            .iter()
            .position(|o| o.choice == choice);

        match choice_index {
            Some(index) => {
                // Get the next dialog ID based on the user's choice
                let next_id = current_dialog.choices[index].next_id.clone();

                if next_id == "exit" {
                    // If the user chooses to exit, end the dialog
                    println!("{}: Thank you for using the dialog system.", agent);
                    break;
                } else {
                    // Otherwise, continue to the next dialog
                    current_id = next_id;
                }

                // Try to predict the user's next choice
                if !predicting_choice {
                    predicting_choice = true;
                }
            }
            None => {
                if predicting_choice {
                    // If user's choice could not be predicted, switch to user mode and repeat the prompt
                    predicting_choice = false;
                    println!("Failed to predict the user's choice.");
                } else {
                    // If no match is found, repeat the prompt
                    println!("{}: I'm sorry, I didn't understand your response.", agent);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(run_dialog()).unwrap();
}
