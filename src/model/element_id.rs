pub(crate) fn element_id_suffix(value: &str) -> String {
    let mut suffix = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_dash = false;
            Some(character.to_ascii_lowercase())
        } else if !previous_dash {
            previous_dash = true;
            Some('-')
        } else {
            None
        };

        if let Some(character) = next {
            suffix.push(character);
        }
    }

    let suffix = suffix.trim_matches('-');
    if suffix.is_empty() {
        String::from("field")
    } else {
        suffix.to_owned()
    }
}
