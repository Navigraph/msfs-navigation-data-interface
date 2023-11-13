pub struct JsonParser;

impl JsonParser {
    pub fn parse(args: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
        let json_string = String::from_utf8(args.to_vec()).unwrap_or_default();
        let trimmed_string = json_string.trim_end_matches(char::from(0));
        let json: serde_json::Value = serde_json::from_str(trimmed_string)?;
        Ok(json)
    }
}
