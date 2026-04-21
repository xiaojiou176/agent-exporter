pub mod archive_index;
pub mod html;
pub mod integration_report;
pub mod json;
pub mod markdown;
pub mod search_report;

pub(crate) const MAX_TOOL_RESULT_LINES: usize = 20;
pub(crate) const OMITTED_TOOL_RESULT_TEXT: &str = "该工具结果过长，已省略";

fn count_lines(value: &str) -> usize {
    if value.is_empty() {
        0
    } else {
        value.lines().count()
    }
}

pub(crate) fn summarize_optional_tool_text(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(summarize_tool_text(trimmed))
}

pub(crate) fn summarize_tool_text(value: &str) -> String {
    let trimmed = value.trim_end();
    if count_lines(trimmed) > MAX_TOOL_RESULT_LINES {
        OMITTED_TOOL_RESULT_TEXT.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn summarize_tool_items(values: &[String]) -> Vec<String> {
    if values.is_empty() {
        return Vec::new();
    }

    let joined = values.join("\n");
    if count_lines(joined.trim_end()) > MAX_TOOL_RESULT_LINES {
        vec![OMITTED_TOOL_RESULT_TEXT.to_string()]
    } else {
        values
            .iter()
            .map(|value| value.trim_end().to_string())
            .collect()
    }
}
