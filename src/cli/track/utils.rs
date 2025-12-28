use std::collections::HashMap;

pub type Kv = HashMap<String, String>;

pub fn parse_kv(tokens: &[String]) -> Kv {
	let mut table = HashMap::new();
	for token in tokens.iter() {
		if let Some((key, value)) = token.split_once('=') {
			table.insert(key.to_string(), value.to_string());
		} else {
			table.insert(token.to_string(), "true".to_string());
		}
	}
	table
}

pub fn resolve_boolean_value(kv: &Kv, key: &str) -> Option<String> {
	if kv.contains_key(key) { Some("true".to_string()) } else { None }
}
