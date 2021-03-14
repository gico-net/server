use uuid::Uuid;

/// Returns a valid Uuid if `id` is not a valid Uuid
pub fn uuid_from_string(id: &String) -> Uuid {
    return match Uuid::parse_str(&id) {
        Ok(x) => x,
        Err(_) => Uuid::parse_str("00000000000000000000000000000000").unwrap(),
    };
}
