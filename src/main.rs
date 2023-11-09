use serde::{Deserialize, Serialize};

fn main() -> Res<()> {
    dotenv().expect(".env could not be loaded");
    // let prompt_response = chat("Why is rust the best programming language ever created?")?;
    // dbg!(&prompt_response);
    // let not_hotdog = "https://i.imgur.com/38RynQm.png";
    // let hotdog = "https://i.imgur.com/FraLxOw.png";
    let hotdog = std::fs::read("hotdogs_or_legs.jpg").expect("hotdog.jpg didn't load");
    let encoded = base64_encode(&hotdog);
    let response = hotdog_or_not_hotdog(&encoded)?;
    // dbg!(&response);
    Ok(())
}

#[allow(unused)]
fn chat(content: &str) -> Res<PromptResponse> {
    let prompt = Prompt {
        model: "gpt-3.5-turbo".into(),
        messages: vec![Message::Chat(ChatMessage {
            role: "user".into(),
            content: content.into(),
        })],
        max_tokens: Some(100),
        response_format: todo!(),
    };
    let response = send_prompt(&prompt)?;

    Ok(response)
}

fn send_prompt(prompt: &Prompt) -> Res<PromptResponse> {
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");
    let request = serde_json::to_string(&prompt).unwrap();
    // dbg!(&request);
    let res = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", openai_api_key))
        .send_string(&request)?;

    let response = res.into_string()?;
    // dbg!(&response);
    let prompt_response = serde_json::from_str::<PromptResponse>(&response)?;

    Ok(prompt_response)
}

fn hotdog_or_not_hotdog(url: &str) -> Res<bool> {
    let content = vec![
        Content {
            r#type: "text".into(),
            text: Some("Is this a hot dog or not a hot dog?".into()),
            image_url: None,
        },
        Content {
            r#type: "image_url".into(),
            image_url: Some(ImageUrl {
                url: format!("data:image/jpeg;base64,{}", url),
            }),
            text: None,
        },
    ];

    let prompt = Prompt {
        model: "gpt-4-vision-preview".into(),
        messages: vec![
            Message::Vision(VisionMessage {
                role: "system".into(),
                content: vec![Content {
                    r#type: "text".into(),
                    text: Some("You are a hotdog detection expert. Is this is an edible hotdog? If it is, then respond with yes if not then respond with no. Only respond with yes or no. Do not end with punctuation.".into()),
                    image_url: None,
                }],
            }),
            Message::Vision(VisionMessage {
                role: "user".into(),
                content,
            }),
        ],
        max_tokens: Some(50),
        response_format: None,
    };

    let response = send_prompt(&prompt)?;

    let choice = response
        .choices
        .iter()
        .nth(0)
        .expect("gpt failed, there wasn't a first choice in the response");

    if let Message::Chat(ChatMessage { content, .. }) = &choice.message {
        dbg!(&content);
        Ok(content.to_lowercase() == "yes")
    } else {
        Err(Error::MismatchMessage)
    }
}

fn dotenv() -> Res<()> {
    let string = std::fs::read_to_string(".env")?;
    for line in string.lines() {
        let parts = line.split("=").map(|part| part.trim()).collect::<Vec<_>>();
        let key = parts.iter().nth(0);
        let value = parts.iter().nth(1);
        if let (Some(key), Some(value)) = (key, value) {
            std::env::set_var(key.to_string(), value.to_string());
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Chat(ChatMessage),
    Vision(VisionMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseFormat {
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisionMessage {
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptResponse {
    choices: Vec<Choice>,
    created: u64,
    id: String,
    model: String,
    object: String,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FinishDetails {
    r#type: String,
    stop: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    //"finish_details\": {\"type\": \"stop\", \"stop\": \"<|fim_suffix|>\"}
    finish_details: Option<FinishDetails>,
    finish_reason: Option<String>,
    index: u32,
    message: Message,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("api error")]
    Api(#[from] ureq::Error),
    #[error("serialization error")]
    Json(#[from] serde_json::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("mismatch chat message in response")]
    MismatchMessage,
}

type Res<T> = Result<T, Error>;

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let base64_chars: Vec<char> =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .chars()
            .collect();

    let mut i = 0;
    while i < data.len() {
        let octet1 = data[i];
        let octet2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let octet3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let index1 = (octet1 >> 2) as usize;
        let index2 = (((octet1 & 0b11) << 4) | (octet2 >> 4)) as usize;
        let index3 = (((octet2 & 0b1111) << 2) | (octet3 >> 6)) as usize;
        let index4 = (octet3 & 0b111111) as usize;

        result.push(base64_chars[index1]);
        result.push(base64_chars[index2]);
        result.push(base64_chars[index3]);
        result.push(base64_chars[index4]);

        i += 3;
    }

    // Add padding if necessary
    let padding = match data.len() % 3 {
        1 => "===",
        2 => "=",
        _ => "",
    };

    result + padding
}
