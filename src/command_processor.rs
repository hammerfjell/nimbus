
use std::sync::Arc;
use std::time::Duration;
use crate::database::Database;
use crate::tokenizer::tokenize_string;

pub async fn process_command(line: &str, db: Arc<Database>) -> String {
    let tokens = tokenize_string(line);

    match tokens.first().map(String::as_str) {
        Some("SET") => process_set_command(&tokens[1..], db).await,
        Some("GET") => process_get_command(&tokens[1..], db).await,
        Some("DEL") => process_del_command(&tokens[1..], db).await,
        _ => "Unsupported command or syntax error.".to_string(),
    }
}

async fn process_set_command(tokens: &[String], db: Arc<Database>) -> String {
    // Initialize optional expiration duration in seconds
    let mut expire_duration_secs: Option<u64> = None;
    let mut new_tokens = Vec::new();

    // Iterate over the tokens to find and handle "EXPIRE"
    let mut tokens_iter = tokens.iter().peekable();
    while let Some(token) = tokens_iter.next() {
        if token == "EXPIRE" {
            if let Some(expire_value) = tokens_iter.next() {
                // Attempt to parse the expiration value
                if let Ok(secs) = expire_value.parse::<u64>() {
                    expire_duration_secs = Some(secs);
                    continue; // Skip adding "EXPIRE" and its value to new_tokens
                } else {
                    return "-ERR Invalid EXPIRE value".to_string();
                }
            } else {
                return "-ERR EXPIRE specified without a value".to_string();
            }
        } else {
            new_tokens.push(token.clone());
            if let Some(next_token) = tokens_iter.peek() {
                if **next_token != "EXPIRE" {
                    new_tokens.push(next_token.to_string());
                }
                tokens_iter.next(); // Advance the iterator to skip adding this in the next iteration
            }
        }
    }

    // Check for even number of arguments excluding "EXPIRE" and its value
    if new_tokens.len() % 2 != 0 {
        return "-ERR SET command requires an even number of arguments.".to_string();
    }

    // Proceed with setting keys and their values, now using expire_duration_secs if available
    for chunk in new_tokens.chunks(2) {
        if let [key, value] = chunk {
            db.set(key.clone(), value.clone(), expire_duration_secs.map(Duration::from_secs)).await;
        }
    }

    "+OK".to_string()
}

async fn process_get_command(tokens: &[String], db: Arc<Database>) -> String {

    let mut results = Vec::new();

    for key in tokens {
        match db.get(key).await {
            Some(value) => results.push(format!("\"{}\"", value.replace("\"", "\\\""))),
            None => results.push("(nil)".to_string()),
        }
    }

    // json
    format!("[{}]", results.join(", "))
}

async fn process_del_command(tokens: &[String], db: Arc<Database>) -> String {
    db.del(tokens).await;
    "+OK".to_string()
}