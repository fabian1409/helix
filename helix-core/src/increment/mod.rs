mod date_time;
mod integer;

pub fn integer(selected_text: &str, amount: i64) -> Option<String> {
    integer::increment(selected_text, amount)
}

pub fn date_time(selected_text: &str, amount: i64) -> Option<String> {
    date_time::increment(selected_text, amount)
}

pub fn bool(selected_text: &str, _amount: i64) -> Option<String> {
    match selected_text {
        "true" => Some("false".to_string()),
        "false" => Some("true".to_string()),
        _ => None,
    }
}
