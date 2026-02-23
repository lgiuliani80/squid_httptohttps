use std::io::{self, BufRead, Write};

/// Squid URL rewrite helper: rewrites http:// URLs to https://
fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        // Squid sends: URL [extras...]
        // We must reply with the rewritten URL (or empty line to keep original)
        let mut parts = line.splitn(2, ' ');
        let url = parts.next().unwrap_or("");
        let extras = parts.next().unwrap_or("");

        let new_url = if url.starts_with("http://") {
            let rewritten = format!("https://{}", &url[7..]);
            eprintln!("[rewriter] {} -> {}", url, rewritten);
            rewritten
        } else {
            eprintln!("[rewriter] pass-through: {}", url);
            url.to_string()
        };

        if extras.is_empty() {
            writeln!(out, "OK store-id={}", new_url).unwrap();
        } else {
            writeln!(out, "OK store-id={} {}", new_url, extras).unwrap();
        }
        out.flush().unwrap();
    }
}
