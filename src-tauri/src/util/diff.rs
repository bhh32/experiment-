use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub status: DiffStatus,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, Serialize)]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub enum LineKind {
    Context,
    Addition,
    Deletion,
}

// Parse a unified diff string into structured FileDiff objects
pub fn parse_unified_diff(raw: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<Hunk> = None;
    let mut old_line: u32 = 0;
    let mut new_line: u32 = 0;

    for line in raw.lines() {
        // New file diff header
        if line.starts_with("diff --git") {
            // Save the previous hunk and file
            if let Some(ref mut file) = current_file {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk);
                }
                files.push(file.clone());
            }
            current_file = Some(FileDiff {
                old_path: String::new(),
                new_path: String::new(),
                status: DiffStatus::Modified,
                hunks: Vec::new(),
            });
            current_hunk = None;
            continue;
        }

        if let Some(ref mut file) = current_file {
            // Parse old/new file paths
            if let Some(path) = line.strip_prefix("--- a/") {
                file.old_path = path.to_string();
                continue;
            }
            if let Some(path) = line.strip_prefix("+++ b/") {
                file.new_path = path.to_string();
                continue;
            }
            if line.starts_with("--- /dev/null") {
                file.status = DiffStatus::Added;
                continue;
            }
            if line.starts_with("+++ /dev/null") {
                file.status = DiffStatus::Deleted;
                continue;
            }

            // Hunk header: @@ -old_start,old_lines +new_start,new_lines @@
            if line.starts_with("@@") {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk);
                }
                if let Some(hunk) = parse_hunk_header(line) {
                    old_line = hunk.old_start;
                    new_line = hunk.new_start;
                    current_hunk = Some(hunk);
                }
                continue;
            }

            // Diff content lines
            if let Some(ref mut hunk) = current_hunk {
                if let Some(content) = line.strip_prefix('+') {
                    hunk.lines.push(DiffLine {
                        kind: LineKind::Addition,
                        content: content.to_string(),
                        old_lineno: None,
                        new_lineno: Some(new_line),
                    });
                    new_line += 1;
                } else if let Some(content) = line.strip_prefix('-') {
                    hunk.lines.push(DiffLine {
                        kind: LineKind::Deletion,
                        content: content.to_string(),
                        old_lineno: Some(old_line),
                        new_lineno: None,
                    });
                    old_line += 1;
                } else if let Some(content) = line.strip_prefix(' ') {
                    hunk.lines.push(DiffLine {
                        kind: LineKind::Context,
                        content: content.to_string(),
                        old_lineno: Some(old_line),
                        new_lineno: Some(new_line),
                    });
                    old_line += 1;
                    new_line += 1;
                }
            }
        }
    }

    // Don't forget the last file
    if let Some(mut file) = current_file {
        if let Some(hunk) = current_hunk {
            file.hunks.push(hunk);
        }
        files.push(file);
    }

    files
}

fn parse_hunk_header(line: &str) -> Option<Hunk> {
    // Format: @@ -old_start,old_lines +new_start,new_lines @@
    let stripped = line.trim_start_matches("@@ ").split(" @@").next()?;
    let parts: Vec<&str> = stripped.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let old = parse_range(parts[0].trim_start_matches('-'))?;
    let new = parse_range(parts[1].trim_start_matches('+'))?;

    Some(Hunk {
        old_start: old.0,
        old_lines: old.1,
        new_start: new.0,
        new_lines: new.1,
        lines: Vec::new(),
    })
}

fn parse_range(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(',').collect();
    let start = parts.first()?.parse().ok()?;
    let lines = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1);
    Some((start, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIFF: &str = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!(\"hello\");
+    println!(\"hello world\");
+    println!(\"goodbye\");
 }";

    #[test]
    fn test_parse_single_file_diff() {
        let files = parse_unified_diff(SAMPLE_DIFF);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].old_path, "src/main.rs");
        assert_eq!(files[0].new_path, "src/main.rs");
    }

    #[test]
    fn test_parse_hunk_lines() {
        let files = parse_unified_diff(SAMPLE_DIFF);
        let hunk = &files[0].hunks[0];

        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.new_start, 1);

        // Should have: 1 context, 1 deletion, 2 additions, 1 context
        let additions = hunk.lines.iter()
            .filter(|l| matches!(l.kind, LineKind::Addition))
            .count();
        let deletions = hunk.lines.iter()
            .filter(|l| matches!(l.kind, LineKind::Deletion))
            .count();

        assert_eq!(additions, 2);
        assert_eq!(deletions, 1);
    }

    #[test]
    fn test_parse_new_file() {
        let diff = "\
diff --git a/new_file.rs b/new_file.rs
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,2 @@
+fn hello() {
+}";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert!(matches!(files[0].status, DiffStatus::Added));
    }

    #[test]
    fn test_parse_empty_diff() {
        let files = parse_unified_diff("");
        assert!(files.is_empty());
    }
}
