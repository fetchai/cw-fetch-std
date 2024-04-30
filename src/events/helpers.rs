pub fn option_to_string<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_string(), |val| val.to_string())
}
