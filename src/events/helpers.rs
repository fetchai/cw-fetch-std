pub fn to_string_or_none<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_string(), |val| val.to_string())
}
