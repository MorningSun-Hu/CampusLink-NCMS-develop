/// 将数据库中以下列形式存储的 UTC 时间统一转换为 RFC3339：
/// - `2026-09-11 11:10:40`
/// - 已含 `T` 的时间原样补 `Z`
///
/// 空字符串或已是带时区的 RFC3339 则保持可解析语义返回。
pub fn to_rfc3339(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    if value.ends_with('Z') || value.contains('+') {
        return value.to_string();
    }
    let normalized = value.replace(' ', "T");
    if normalized.contains('T') {
        format!("{}Z", normalized)
    } else {
        normalized
    }
}

pub fn to_rfc3339_opt(value: Option<&str>) -> Option<String> {
    value
        .filter(|v| !v.is_empty())
        .map(to_rfc3339)
}
