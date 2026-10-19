use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::fs::{descriptor::FilePolicy, FileSnapshot, HeldDirectory};
use walkdir::WalkDir;

use super::types::{
    FileSanitizationReport, SanitizationCategory, SanitizationFinding, SanitizationReport,
    SanitizerConfig,
};
use crate::error::AppError;

pub struct Sanitizer {
    config: SanitizerConfig,
}

impl Sanitizer {
    pub fn new(config: SanitizerConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SanitizerConfig::default())
    }

    pub fn scrub_str(&self, input: &str) -> String {
        let mut text = input.to_string();

        if self.config.scrub_secrets {
            text = self.scrub_secrets_text(&text);
        }
        if self.config.scrub_user_paths {
            text = self.scrub_user_paths_text(&text);
        }
        if self.config.scrub_emails {
            text = self.scrub_emails_text(&text);
        }
        if self.config.scrub_ips {
            text = self.scrub_ips_text(&text);
        }
        if self.config.scrub_hostnames {
            text = self.scrub_hostnames_text(&text);
        }
        if self.config.scrub_proprietary {
            text = self.scrub_proprietary_text(&text);
        }

        text
    }

    pub fn scan_str(&self, input: &str) -> Vec<SanitizationFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in input.lines().enumerate() {
            let line_num = line_idx + 1;

            if self.config.scrub_secrets {
                let trimmed = line.trim();
                if trimmed.starts_with("-----BEGIN ") && trimmed.ends_with("PRIVATE KEY-----") {
                    findings.push(SanitizationFinding {
                        category: SanitizationCategory::Secret,
                        rule_name: "private_key".to_string(),
                        line_number: line_num,
                        start_col: 1,
                        snippet: "[PRIVATE KEY BLOCK]".to_string(),
                    });
                }
                if let Some(col) = line.find("Bearer ") {
                    let token_part = &line[col + 7..];
                    if token_part.len() > 15
                        && !token_part.starts_with("secret-token")
                        && !token_part.starts_with("test-token")
                        && !token_part.starts_with("example-token")
                    {
                        findings.push(SanitizationFinding {
                            category: SanitizationCategory::Secret,
                            rule_name: "bearer_token".to_string(),
                            line_number: line_num,
                            start_col: col + 1,
                            snippet: "Bearer ...".to_string(),
                        });
                    }
                }
            }

            if self.config.scrub_user_paths {
                for prefix in &["/Users/", "/home/"] {
                    let mut search_from = 0;
                    while let Some(idx) = line[search_from..].find(prefix) {
                        let abs_idx = search_from + idx;
                        let after_prefix = abs_idx + prefix.len();
                        let username_end = line[after_prefix..]
                            .find(|c: char| {
                                c == '/'
                                    || c == ' '
                                    || c == '"'
                                    || c == '\''
                                    || c == ':'
                                    || c == '\n'
                                    || c == '`'
                            })
                            .map(|end| after_prefix + end)
                            .unwrap_or(line.len());
                        if username_end > after_prefix {
                            let username = &line[after_prefix..username_end];
                            if !is_generic_system_name(username)
                                && !username.contains('.')
                                && line[username_end..].starts_with('/')
                            {
                                findings.push(SanitizationFinding {
                                    category: SanitizationCategory::Pii,
                                    rule_name: "user_home_path".to_string(),
                                    line_number: line_num,
                                    start_col: abs_idx + 1,
                                    snippet: safe_snippet(line, abs_idx, 30),
                                });
                            }
                        }
                        search_from = after_prefix;
                    }
                }
                if let Some(col) = self.find_data_home(line) {
                    findings.push(SanitizationFinding {
                        category: SanitizationCategory::Pii,
                        rule_name: "data_user_home_path".to_string(),
                        line_number: line_num,
                        start_col: col + 1,
                        snippet: safe_snippet(line, col, 30),
                    });
                }
            }

            if self.config.scrub_emails {
                for (col, email) in self.extract_emails(line) {
                    findings.push(SanitizationFinding {
                        category: SanitizationCategory::Pii,
                        rule_name: "email_address".to_string(),
                        line_number: line_num,
                        start_col: col + 1,
                        snippet: email,
                    });
                }
            }

            if self.config.scrub_ips {
                for (col, ip) in self.extract_ips(line) {
                    findings.push(SanitizationFinding {
                        category: SanitizationCategory::MachineIdentifiable,
                        rule_name: "ip_address".to_string(),
                        line_number: line_num,
                        start_col: col + 1,
                        snippet: ip,
                    });
                }
            }

            if self.config.scrub_hostnames {
                for (col, host) in self.extract_node_hosts(line) {
                    findings.push(SanitizationFinding {
                        category: SanitizationCategory::MachineIdentifiable,
                        rule_name: "machine_hostname".to_string(),
                        line_number: line_num,
                        start_col: col + 1,
                        snippet: host,
                    });
                }
            }

            if self.config.scrub_proprietary {
                let lower_line = line.to_ascii_lowercase();
                for term in &self.config.custom_proprietary_terms {
                    let term_lower = term.to_ascii_lowercase();
                    if let Some(col) = lower_line.find(&term_lower) {
                        findings.push(SanitizationFinding {
                            category: SanitizationCategory::Proprietary,
                            rule_name: format!("proprietary_term_{term}"),
                            line_number: line_num,
                            start_col: col + 1,
                            snippet: term.clone(),
                        });
                    }
                }
            }
        }

        findings.retain(|f| {
            !self
                .config
                .allowlisted_patterns
                .iter()
                .any(|p| f.snippet.contains(p) || p.contains(&f.snippet))
        });

        findings
    }

    pub fn scan_path(&self, root: &Path) -> Result<SanitizationReport, AppError> {
        let mut report = SanitizationReport {
            scanned_files: 0,
            total_findings: 0,
            file_reports: Vec::new(),
            clean: true,
        };

        let root_path = checked_root(root)?;
        let root = root_path.as_path();
        if root.is_file() {
            let findings = self.scan_file(root)?;
            let total = findings.len();
            if !findings.is_empty() {
                report.clean = false;
                report.total_findings += total;
                report.file_reports.push(FileSanitizationReport {
                    path: root.to_path_buf(),
                    findings,
                });
            }
            report.scanned_files = 1;
            return Ok(report);
        }

        for entry in WalkDir::new(root)
            .follow_links(false)
            .follow_root_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored_scan_dir(e.file_name()))
        {
            let entry = entry
                .map_err(|e| AppError::io("sanitize.walk", "read directory entry", e.into()))?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if is_text_candidate(path) {
                    let findings = self.scan_file(path)?;
                    report.scanned_files += 1;
                    if !findings.is_empty() {
                        report.clean = false;
                        report.total_findings += findings.len();
                        report.file_reports.push(FileSanitizationReport {
                            path: path.to_path_buf(),
                            findings,
                        });
                    }
                }
            }
        }

        Ok(report)
    }

    pub fn scrub_file(&self, path: &Path, in_place: bool) -> Result<String, AppError> {
        let (directory, relative, bytes, snapshot) = read_regular_file(path)?;
        let content = String::from_utf8(bytes).map_err(|error| {
            AppError::invalid_input("sanitize.encoding", format!("input must be UTF-8: {error}"))
        })?;
        let scrubbed = self.scrub_str(&content);
        if in_place && scrubbed != content {
            directory.compare_and_replace_public_regular_file(
                &relative,
                scrubbed.as_bytes(),
                Some(&snapshot),
                "sanitizer input",
                usize::MAX,
            )?;
        }
        Ok(scrubbed)
    }

    pub fn scrub_path(&self, root: &Path) -> Result<usize, AppError> {
        let root_path = checked_root(root)?;
        let root = root_path.as_path();
        if root.is_file() {
            let (_, _, bytes, _) = read_regular_file(root)?;
            let scrubbed = self.scrub_file(root, true)?;
            return Ok(usize::from(scrubbed.as_bytes() != bytes));
        }
        // Inspect the selected tree before writing. A symlink is an error rather
        // than a route to another tree; every eventual write reopens its held path.
        let mut files = Vec::new();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .follow_root_links(false)
            .into_iter()
            .filter_entry(|entry| !is_ignored_scan_dir(entry.file_name()))
        {
            let entry = entry.map_err(|error| {
                AppError::io("sanitize.walk", "read directory entry", error.into())
            })?;
            if entry.file_type().is_symlink() {
                return Err(AppError::invalid_input(
                    "sanitize.symlink",
                    format!("refusing symlink {}", entry.path().display()),
                ));
            }
            if entry.file_type().is_file()
                && is_text_candidate(entry.path())
                && !is_sanitizer_source(entry.path())
            {
                files.push(entry.into_path());
            }
        }
        let mut count = 0;
        for path in files {
            let (_, _, bytes, _) = read_regular_file(&path)?;
            // Binary files are outside the text sanitizer's input contract.
            let Ok(content) = std::str::from_utf8(&bytes) else {
                continue;
            };
            if self.scrub_str(content) != content {
                self.scrub_file(&path, true)?;
                count += 1;
            }
        }
        Ok(count)
    }

    fn scan_file(&self, path: &Path) -> Result<Vec<SanitizationFinding>, AppError> {
        let (_, _, bytes, _) = read_regular_file(path)?;
        if is_sanitizer_source(path) {
            return Ok(Vec::new());
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            return Ok(Vec::new());
        };
        Ok(self.scan_str(text))
    }

    // --- Private scrubbers ---

    fn scrub_secrets_text(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut in_key_block = false;

        for segment in text.split_inclusive('\n') {
            let (line, ending) = if let Some(line) = segment.strip_suffix("\r\n") {
                (line, "\r\n")
            } else if let Some(line) = segment.strip_suffix('\n') {
                (line, "\n")
            } else {
                (segment, "")
            };
            let trimmed = line.trim();
            if trimmed.starts_with("-----BEGIN ") && trimmed.ends_with("PRIVATE KEY-----") {
                in_key_block = true;
                out.push_str("[REDACTED_PRIVATE_KEY]");
                out.push_str(ending);
                continue;
            }
            if in_key_block {
                if trimmed.starts_with("-----END ") && trimmed.ends_with("PRIVATE KEY-----") {
                    in_key_block = false;
                }
                continue;
            }

            let mut scrubbed_line = line.to_string();
            if let Some(pos) = scrubbed_line.find("Bearer ") {
                let token_start = pos + 7;
                let token_part = &scrubbed_line[token_start..];
                if !token_part.starts_with("secret-token")
                    && !token_part.starts_with("test-token")
                    && !token_part.starts_with("example-token")
                {
                    let token_end = token_part
                        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ';')
                        .map(|end| token_start + end)
                        .unwrap_or(scrubbed_line.len());
                    if token_end > token_start + 8 {
                        scrubbed_line.replace_range(token_start..token_end, "[REDACTED_TOKEN]");
                    }
                }
            }

            out.push_str(&scrubbed_line);
            out.push_str(ending);
        }

        out
    }

    fn scrub_user_paths_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        let mut search_from = 0;
        while let Some((start, end)) = self.find_data_home_range(&result[search_from..]) {
            let abs_start = search_from + start;
            let abs_end = search_from + end;
            result.replace_range(abs_start..abs_end, "[REDACTED_USER_HOME]");
            search_from = abs_start + "[REDACTED_USER_HOME]".len();
        }

        for prefix in &["/Users/", "/home/"] {
            let mut search_from = 0;
            while let Some(idx) = result[search_from..].find(prefix) {
                let abs_idx = search_from + idx;
                let after_prefix = abs_idx + prefix.len();
                let username_end = result[after_prefix..]
                    .find(|c: char| {
                        c == '/' || c == ' ' || c == '"' || c == '\'' || c == ':' || c == '\n'
                    })
                    .map(|end| after_prefix + end)
                    .unwrap_or(result.len());

                if username_end > after_prefix {
                    let username = &result[after_prefix..username_end];
                    if !is_generic_system_name(username) {
                        result.replace_range(abs_idx..username_end, "[REDACTED_USER_HOME]");
                        search_from = abs_idx + "[REDACTED_USER_HOME]".len();
                        continue;
                    }
                }
                search_from = after_prefix;
            }
        }

        result
    }

    fn scrub_emails_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        let mut search_from = 0;

        while search_from < result.len() {
            let slice = &result[search_from..];
            let Some(at_idx) = slice.find('@') else {
                break;
            };

            let before = &slice[..at_idx];
            let after = &slice[at_idx + 1..];

            let start = before
                .char_indices()
                .rfind(|(_, c)| {
                    !c.is_ascii_alphanumeric()
                        && *c != '.'
                        && *c != '+'
                        && *c != '-'
                        && *c != '_'
                        && *c != '%'
                })
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);

            let mut end_opt = None;
            for (i, c) in after.char_indices() {
                if !c.is_ascii_alphanumeric() && c != '.' && c != '-' {
                    end_opt = Some(i);
                    break;
                }
            }
            let mut end = end_opt.unwrap_or(after.len());

            while end > 0 {
                let prev_char = after[..end].chars().next_back().unwrap();
                if prev_char == '.' || prev_char == '-' {
                    end -= 1;
                } else {
                    break;
                }
            }

            let local_part = &before[start..];
            let domain_part = &after[..end];

            if !local_part.is_empty()
                && domain_part.contains('.')
                && !domain_part.starts_with('.')
                && !domain_part.ends_with('.')
                && !is_rfc2606_example_domain(domain_part)
                && domain_part
                    .split('.')
                    .next_back()
                    .map(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
                    .unwrap_or(false)
            {
                let abs_start = search_from + start;
                let abs_end = search_from + at_idx + 1 + end;
                result.replace_range(abs_start..abs_end, "[REDACTED_EMAIL]");
                search_from = abs_start + "[REDACTED_EMAIL]".len();
            } else {
                search_from += at_idx + 1;
            }
        }

        result
    }

    fn scrub_ips_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        let mut search_from = 0;

        while search_from < result.len() {
            let slice = &result[search_from..];
            let mut found = None;

            for (idx, _) in slice.char_indices() {
                if is_ip_start(slice, idx) {
                    if let Some(len) = parse_ipv4_len(&slice[idx..]) {
                        found = Some((idx, len));
                        break;
                    }
                }
            }

            if let Some((idx, len)) = found {
                let abs_start = search_from + idx;
                let abs_end = abs_start + len;
                result.replace_range(abs_start..abs_end, "[REDACTED_IP]");
                search_from = abs_start + "[REDACTED_IP]".len();
            } else {
                break;
            }
        }

        result
    }

    fn scrub_hostnames_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        let known_hosts = [
            "devx_b200_dbg",
            "devx_b200_tiger",
            "jumper_general_us_proxy",
        ];

        for host in known_hosts {
            result = result.replace(host, "[REDACTED_HOST]");
        }

        let mut search_from = 0;
        while search_from < result.len() {
            let slice = &result[search_from..];
            if let Some((idx, len)) = find_node_hostname(slice) {
                let abs_start = search_from + idx;
                let abs_end = abs_start + len;
                result.replace_range(abs_start..abs_end, "[REDACTED_HOST]");
                search_from = abs_start + "[REDACTED_HOST]".len();
            } else {
                break;
            }
        }

        result
    }

    fn scrub_proprietary_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        for term in &self.config.custom_proprietary_terms {
            if term.is_empty() {
                continue;
            }
            let mut search_from = 0;
            while search_from < result.len() {
                let lower_slice = result[search_from..].to_ascii_lowercase();
                let lower_term = term.to_ascii_lowercase();
                if let Some(idx) = lower_slice.find(&lower_term) {
                    let abs_start = search_from + idx;
                    let abs_end = abs_start + term.len();
                    result.replace_range(abs_start..abs_end, "[REDACTED_ORG]");
                    search_from = abs_start + "[REDACTED_ORG]".len();
                } else {
                    break;
                }
            }
        }

        result
    }

    // --- Helpers ---

    fn find_data_home(&self, line: &str) -> Option<usize> {
        self.find_data_home_range(line).map(|(start, _)| start)
    }

    fn find_data_home_range(&self, line: &str) -> Option<(usize, usize)> {
        let mut search_idx = 0;
        while let Some(idx) = line[search_idx..].find("/data") {
            let abs_data = search_idx + idx;
            let after_data = abs_data + 5;
            let digit_end = line[after_data..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|pos| after_data + pos)
                .unwrap_or(after_data);

            if line[digit_end..].starts_with("/home/") {
                let user_start = digit_end + 6;
                let user_end = line[user_start..]
                    .find(|c: char| {
                        c == '/' || c == ' ' || c == '"' || c == '\'' || c == ':' || c == '\n'
                    })
                    .map(|pos| user_start + pos)
                    .unwrap_or(line.len());
                if user_end > user_start {
                    return Some((abs_data, user_end));
                }
            }
            search_idx = abs_data + 5;
        }
        None
    }

    fn extract_emails(&self, line: &str) -> Vec<(usize, String)> {
        let mut emails = Vec::new();
        let mut search_from = 0;

        while search_from < line.len() {
            let slice = &line[search_from..];
            let Some(at_idx) = slice.find('@') else {
                break;
            };

            let before = &slice[..at_idx];
            let after = &slice[at_idx + 1..];

            let start = before
                .char_indices()
                .rfind(|(_, c)| {
                    !c.is_ascii_alphanumeric()
                        && *c != '.'
                        && *c != '+'
                        && *c != '-'
                        && *c != '_'
                        && *c != '%'
                })
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);

            let mut end_opt = None;
            for (i, c) in after.char_indices() {
                if !c.is_ascii_alphanumeric() && c != '.' && c != '-' {
                    end_opt = Some(i);
                    break;
                }
            }
            let mut end = end_opt.unwrap_or(after.len());

            while end > 0 {
                let prev_char = after[..end].chars().next_back().unwrap();
                if prev_char == '.' || prev_char == '-' {
                    end -= 1;
                } else {
                    break;
                }
            }

            let local_part = &before[start..];
            let domain_part = &after[..end];

            if !local_part.is_empty()
                && domain_part.contains('.')
                && !domain_part.starts_with('.')
                && !domain_part.ends_with('.')
                && !is_rfc2606_example_domain(domain_part)
                && domain_part
                    .split('.')
                    .next_back()
                    .map(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
                    .unwrap_or(false)
            {
                emails.push((search_from + start, format!("{local_part}@{domain_part}")));
                search_from += at_idx + 1 + end;
            } else {
                search_from += at_idx + 1;
            }
        }

        emails
    }

    fn extract_ips(&self, line: &str) -> Vec<(usize, String)> {
        let mut ips = Vec::new();
        let mut search_from = 0;

        while search_from < line.len() {
            let slice = &line[search_from..];
            let mut found = None;

            for (idx, _) in slice.char_indices() {
                if is_ip_start(slice, idx) {
                    if let Some(len) = parse_ipv4_len(&slice[idx..]) {
                        found = Some((idx, len));
                        break;
                    }
                }
            }

            if let Some((idx, len)) = found {
                let ip_str = slice[idx..idx + len].to_string();
                ips.push((search_from + idx, ip_str));
                search_from += idx + len;
            } else {
                break;
            }
        }

        ips
    }

    fn extract_node_hosts(&self, line: &str) -> Vec<(usize, String)> {
        let mut hosts = Vec::new();
        for known in &[
            "devx_b200_dbg",
            "devx_b200_tiger",
            "jumper_general_us_proxy",
        ] {
            if let Some(col) = line.find(known) {
                hosts.push((col, known.to_string()));
            }
        }
        if let Some((col, len)) = find_node_hostname(line) {
            hosts.push((col, line[col..col + len].to_string()));
        }
        hosts
    }
}

fn is_generic_system_name(name: &str) -> bool {
    matches!(
        name,
        "bin"
            | "lib"
            | "local"
            | "share"
            | "src"
            | "etc"
            | "var"
            | "usr"
            | "opt"
            | "runner"
            | "root"
            | "default"
            | "public"
            | "example"
            | "someone"
            | "alice"
            | "user"
    )
}

fn is_ip_start(s: &str, idx: usize) -> bool {
    if idx == 0 {
        return true;
    }
    let prev = s[..idx].chars().next_back().unwrap();
    !prev.is_alphanumeric()
        && prev != '.'
        && prev != '/'
        && prev != '§'
        && prev != 'v'
        && prev != 'V'
        && prev != '#'
}

fn is_rfc2606_example_domain(domain: &str) -> bool {
    domain.ends_with(".invalid")
        || domain.ends_with(".example")
        || domain.ends_with(".test")
        || domain.ends_with(".localhost")
        || domain == "example.com"
        || domain == "example.org"
        || domain == "example.net"
}

fn parse_ipv4_len(s: &str) -> Option<usize> {
    let mut parts = s.splitn(4, '.');
    let p0 = parts.next()?;
    let p1 = parts.next()?;
    let p2 = parts.next()?;
    let rest = parts.next()?;

    let (p3, remaining_len) = extract_numeric_part(rest)?;

    let v0: u8 = p0.parse().ok()?;
    let v1: u8 = p1.parse().ok()?;
    let v2: u8 = p2.parse().ok()?;
    let v3: u8 = p3.parse().ok()?;

    if p0.len() > 3 || p1.len() > 3 || p2.len() > 3 || p3.len() > 3 {
        return None;
    }

    // Ignore loopback (127.x.x.x) and unassigned 0.0.0.0
    if v0 == 127 || (v0 == 0 && v1 == 0 && v2 == 0 && v3 == 0) {
        return None;
    }

    // Verify it is followed by non-alphanumeric or end of string
    let total_len = p0.len() + 1 + p1.len() + 1 + p2.len() + 1 + remaining_len;
    if total_len < s.len() {
        let next_char = s[total_len..].chars().next().unwrap();
        if next_char.is_alphanumeric() {
            return None;
        }
    }

    let _ = (v0, v1, v2, v3);
    Some(total_len)
}

fn extract_numeric_part(s: &str) -> Option<(&str, usize)> {
    let len = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if len == 0 || len > 3 {
        None
    } else {
        Some((&s[..len], len))
    }
}

fn find_node_hostname(s: &str) -> Option<(usize, usize)> {
    // Pattern: n[1-3 digits]-[1-3 digits]-[1-3 digits]
    let bytes = s.as_bytes();
    if bytes.len() < 8 {
        return None;
    }

    for i in 0..bytes.len().saturating_sub(7) {
        if bytes[i] == b'n' {
            let mut j = i + 1;
            let start1 = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let len1 = j - start1;
            if len1 == 0 || len1 > 3 || j >= bytes.len() || bytes[j] != b'-' {
                continue;
            }
            j += 1;
            let start2 = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let len2 = j - start2;
            if len2 == 0 || len2 > 3 || j >= bytes.len() || bytes[j] != b'-' {
                continue;
            }
            j += 1;
            let start3 = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let len3 = j - start3;
            if (1..=3).contains(&len3) {
                return Some((i, j - i));
            }
        }
    }
    None
}

fn is_ignored_scan_dir(name: &std::ffi::OsStr) -> bool {
    let s = name.to_string_lossy();
    matches!(
        s.as_ref(),
        ".git"
            | "target"
            | ".build"
            | "node_modules"
            | ".lake"
            | ".obsidian"
            | "dist"
            | "output"
            | "tmp"
            | "evidence"
            | ".worktrees"
            | ".harp"
            | ".sources"
            | ".vite"
            | "local"
    )
}

fn is_text_candidate(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if matches!(
            name,
            "Cargo.lock"
                | "pnpm-lock.yaml"
                | "package-lock.json"
                | "yarn.lock"
                | "lake-manifest.json"
        ) {
            return false;
        }
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(
        ext,
        "md" | "txt"
            | "json"
            | "jsonl"
            | "toml"
            | "yaml"
            | "yml"
            | "rs"
            | "py"
            | "sh"
            | "tsv"
            | "csv"
            | "sql"
    )
}

fn safe_snippet(s: &str, start_byte: usize, max_chars: usize) -> String {
    if start_byte >= s.len() {
        return String::new();
    }
    let mut safe_start = start_byte;
    while safe_start < s.len() && !s.is_char_boundary(safe_start) {
        safe_start += 1;
    }
    s[safe_start..].chars().take(max_chars).collect()
}

fn absolute_input(path: &Path) -> Result<PathBuf, AppError> {
    let absolute = std::path::absolute(path)
        .map_err(|error| AppError::io("sanitize.path", "resolve input path", error))?;
    if absolute
        .components()
        .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(AppError::invalid_input(
            "sanitize.path",
            "input path must not contain parent traversal",
        ));
    }
    Ok(absolute)
}

fn checked_root(path: &Path) -> Result<PathBuf, AppError> {
    let absolute = absolute_input(path)?;
    let root = HeldDirectory::open(Path::new("/"), "sanitizer filesystem root")?;
    let relative = absolute.strip_prefix("/").map_err(|_| {
        AppError::invalid_input(
            "sanitize.path",
            "input must use an absolute filesystem path",
        )
    })?;
    let metadata = fs::symlink_metadata(&absolute)
        .map_err(|error| AppError::io("sanitize.path", "inspect input", error))?;
    if metadata.is_dir() {
        root.anchored_directory("sanitizer root")?
            .walk(relative, "sanitizer root")?;
    } else {
        read_regular_file(&absolute)?;
    }
    Ok(absolute)
}

fn read_regular_file(
    path: &Path,
) -> Result<(HeldDirectory, PathBuf, Vec<u8>, FileSnapshot), AppError> {
    let absolute = absolute_input(path)?;
    let directory = HeldDirectory::open(Path::new("/"), "sanitizer filesystem root")?;
    let relative = absolute
        .strip_prefix("/")
        .map_err(|_| {
            AppError::invalid_input(
                "sanitize.path",
                "input must use an absolute filesystem path",
            )
        })?
        .to_path_buf();
    let (parent, name) = directory
        .anchored_directory("sanitizer input")?
        .walk_parent(&relative, "sanitizer input")?;
    let (bytes, snapshot) = parent
        .read_optional_bounded(&name, "sanitizer input", usize::MAX, FilePolicy::REGULAR)?
        .ok_or_else(|| AppError::invalid_input("sanitize.missing", "sanitizer input is missing"))?;
    Ok((directory, relative, bytes, snapshot))
}

fn is_sanitizer_source(path: &Path) -> bool {
    path.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sanitize"))
}
