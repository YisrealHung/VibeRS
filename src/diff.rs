#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffOpKind {
    Equal,
    Delete,
    Insert,
}

#[derive(Clone, Debug)]
pub struct DiffLine {
    pub kind: DiffOpKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct DiffHunk {
    pub id: usize,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

/// Compute unified diff hunks (with up to 3 lines of surrounding context)
pub fn compute_hunks(old_text: &str, new_text: &str) -> Vec<DiffHunk> {
    if old_text == new_text {
        return Vec::new();
    }

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let all_ops = diff_lines(&old_lines, &new_lines);

    let context_radius = 3usize;
    let n = all_ops.len();
    let mut changed_indices = Vec::new();
    for (i, op) in all_ops.iter().enumerate() {
        if op.kind != DiffOpKind::Equal {
            changed_indices.push(i);
        }
    }

    if changed_indices.is_empty() {
        return Vec::new();
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for idx in changed_indices {
        let start = idx.saturating_sub(context_radius);
        let end = (idx + context_radius + 1).min(n);
        if let Some(last) = ranges.last_mut() {
            if start <= last.1 {
                last.1 = end;
                continue;
            }
        }
        ranges.push((start, end));
    }

    let mut hunks = Vec::new();
    for (hunk_idx, (start, end)) in ranges.into_iter().enumerate() {
        let slice = &all_ops[start..end];
        let old_start = slice.iter().find_map(|l| l.old_line).unwrap_or(1);
        let new_start = slice.iter().find_map(|l| l.new_line).unwrap_or(1);
        let old_count = slice.iter().filter(|l| l.old_line.is_some()).count();
        let new_count = slice.iter().filter(|l| l.new_line.is_some()).count();

        hunks.push(DiffHunk {
            id: hunk_idx + 1,
            old_start,
            old_count,
            new_start,
            new_count,
            lines: slice.to_vec(),
        });
    }

    hunks
}

/// Revert a specific hunk in `new_text` back to what `old_text` had
pub fn revert_hunk_in_text(old_text: &str, new_text: &str, target_hunk_id: usize) -> String {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let all_ops = diff_lines(&old_lines, &new_lines);

    let context_radius = 3usize;
    let n = all_ops.len();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (i, op) in all_ops.iter().enumerate() {
        if op.kind != DiffOpKind::Equal {
            let start = i.saturating_sub(context_radius);
            let end = (i + context_radius + 1).min(n);
            if let Some(last) = ranges.last_mut() {
                if start <= last.1 {
                    last.1 = end;
                    continue;
                }
            }
            ranges.push((start, end));
        }
    }

    let target_range = ranges
        .get(target_hunk_id.saturating_sub(1))
        .copied()
        .unwrap_or((0, 0));

    let mut out = Vec::new();
    for (idx, op) in all_ops.iter().enumerate() {
        let in_target = idx >= target_range.0 && idx < target_range.1;
        match op.kind {
            DiffOpKind::Equal => out.push(op.text.as_str()),
            DiffOpKind::Delete => {
                if in_target {
                    out.push(op.text.as_str());
                }
            }
            DiffOpKind::Insert => {
                if !in_target {
                    out.push(op.text.as_str());
                }
            }
        }
    }

    let mut result = out.join("\n");
    if new_text.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Accept a specific hunk by updating the snapshot (advancing it to match new_text for this hunk)
pub fn accept_hunk_in_snapshot(old_text: &str, new_text: &str, target_hunk_id: usize) -> String {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let all_ops = diff_lines(&old_lines, &new_lines);

    let context_radius = 3usize;
    let n = all_ops.len();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (i, op) in all_ops.iter().enumerate() {
        if op.kind != DiffOpKind::Equal {
            let start = i.saturating_sub(context_radius);
            let end = (i + context_radius + 1).min(n);
            if let Some(last) = ranges.last_mut() {
                if start <= last.1 {
                    last.1 = end;
                    continue;
                }
            }
            ranges.push((start, end));
        }
    }

    let target_range = ranges
        .get(target_hunk_id.saturating_sub(1))
        .copied()
        .unwrap_or((0, 0));

    let mut out = Vec::new();
    for (idx, op) in all_ops.iter().enumerate() {
        let in_target = idx >= target_range.0 && idx < target_range.1;
        match op.kind {
            DiffOpKind::Equal => out.push(op.text.as_str()),
            DiffOpKind::Delete => {
                if !in_target {
                    out.push(op.text.as_str());
                }
            }
            DiffOpKind::Insert => {
                if in_target {
                    out.push(op.text.as_str());
                }
            }
        }
    }

    let mut result = out.join("\n");
    if new_text.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn diff_lines(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let mut prefix = 0;
    while prefix < old.len() && prefix < new.len() && old[prefix] == new[prefix] {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < (old.len() - prefix)
        && suffix < (new.len() - prefix)
        && old[old.len() - 1 - suffix] == new[new.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let old_mid = &old[prefix..old.len() - suffix];
    let new_mid = &new[prefix..new.len() - suffix];

    let mut result = Vec::with_capacity(old.len() + new.len());

    for i in 0..prefix {
        result.push(DiffLine {
            kind: DiffOpKind::Equal,
            old_line: Some(i + 1),
            new_line: Some(i + 1),
            text: old[i].to_string(),
        });
    }

    let m = old_mid.len();
    let n = new_mid.len();
    let width = n + 1;
    let mut dp = vec![0u32; (m + 1) * width];

    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i * width + j] = if old_mid[i] == new_mid[j] {
                1 + dp[(i + 1) * width + (j + 1)]
            } else {
                dp[(i + 1) * width + j].max(dp[i * width + (j + 1)])
            };
        }
    }

    let mut i = 0;
    let mut j = 0;
    while i < m && j < n {
        if old_mid[i] == new_mid[j] {
            result.push(DiffLine {
                kind: DiffOpKind::Equal,
                old_line: Some(prefix + i + 1),
                new_line: Some(prefix + j + 1),
                text: old_mid[i].to_string(),
            });
            i += 1;
            j += 1;
        } else if dp[(i + 1) * width + j] >= dp[i * width + (j + 1)] {
            result.push(DiffLine {
                kind: DiffOpKind::Delete,
                old_line: Some(prefix + i + 1),
                new_line: None,
                text: old_mid[i].to_string(),
            });
            i += 1;
        } else {
            result.push(DiffLine {
                kind: DiffOpKind::Insert,
                old_line: None,
                new_line: Some(prefix + j + 1),
                text: new_mid[j].to_string(),
            });
            j += 1;
        }
    }

    while i < m {
        result.push(DiffLine {
            kind: DiffOpKind::Delete,
            old_line: Some(prefix + i + 1),
            new_line: None,
            text: old_mid[i].to_string(),
        });
        i += 1;
    }

    while j < n {
        result.push(DiffLine {
            kind: DiffOpKind::Insert,
            old_line: None,
            new_line: Some(prefix + j + 1),
            text: new_mid[j].to_string(),
        });
        j += 1;
    }

    let old_suffix_start = old.len() - suffix;
    let new_suffix_start = new.len() - suffix;
    for k in 0..suffix {
        result.push(DiffLine {
            kind: DiffOpKind::Equal,
            old_line: Some(old_suffix_start + k + 1),
            new_line: Some(new_suffix_start + k + 1),
            text: old[old_suffix_start + k].to_string(),
        });
    }

    result
}

