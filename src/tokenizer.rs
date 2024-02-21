pub fn tokenize_string(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        // Check if the next character should be escaped
        if escape_next {
            current_token.push(c);
            escape_next = false;
            chars.next();
            continue;
        }

        match c {
            // When a backslash is encountered, and we're inside quotes, escape the next character
            '\\' if in_quotes => {
                escape_next = true;
                chars.next();
            },
            // Toggle in_quotes flag when a quote is encountered
            '"' => {
                in_quotes = !in_quotes;
                chars.next();
                // Do not skip the quote if it's the start or end of a quoted token
                if !in_quotes || current_token.is_empty() {
                    continue;
                }
            },
            // Space characters delimit tokens unless we're inside a quoted string
            ' ' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            // Any other character should be added to the current token
            _ => {
                current_token.push(c);
                chars.next();
            }
        }
    }

    // Add the last token, if any
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}