//! Fetch MTGJSON data into the data dir.
//!
//! The bulk download is delegated to `curl` (ubiquitous on macOS, in the dev
//! shell, and wrapped onto PATH for the Nix build), so the Rust binary carries
//! no TLS/HTTP dependency tree. Decompression is done in-process with flate2.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

const BASE: &str = "https://mtgjson.com/api/v5";
const CUBE_PAGE: usize = 96;

pub fn download(data_dir: &Path, all: bool, force: bool) -> Result<()> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;

    if let Ok(meta) = curl_text(&format!("{BASE}/Meta.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&meta) {
            if let Some(ver) = v["data"]["version"].as_str() {
                println!("MTGJSON build {ver}");
            }
        }
    }

    fetch_gz("AtomicCards", data_dir, force)?;
    if all {
        fetch_gz("AllPrintings", data_dir, force)?;
    }

    println!("Done. Next: mtgbrain build");
    Ok(())
}

fn fetch_gz(name: &str, data_dir: &Path, force: bool) -> Result<()> {
    let json_path = data_dir.join(format!("{name}.json"));
    if json_path.exists() && !force {
        println!(
            "  {name}: present ({}); pass --force to refresh",
            json_path.display()
        );
        return Ok(());
    }

    let gz_path = data_dir.join(format!("{name}.json.gz"));
    let url = format!("{BASE}/{name}.json.gz");
    println!("  {name}: downloading {url}");
    curl_to_file(&url, &gz_path)?;

    println!("  {name}: inflating -> {}", json_path.display());
    inflate(&gz_path, &json_path)?;

    let size = std::fs::metadata(&json_path)?.len();
    println!("  {name}: ready ({:.1} MB)", size as f64 / 1e6);
    Ok(())
}

/// Fetch CubeCobra card Elo ratings (a cube-draft-pick signal complementary to
/// EDHREC) into `cubecobra_elo.jsonl`. The `topcards` API is paginated 96/page,
/// sorted by Elo descending; `f=cmc>=0` matches every non-token card. Each line
/// is `{name_lower, oracle_id, elo, cube_count, pick_count, popularity}`; build
/// joins it to `cards` on oracle_id (falling back to name).
pub fn cubecobra(data_dir: &Path, max_pages: Option<usize>) -> Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let out_path = data_dir.join("cubecobra_elo.jsonl");
    let tmp = PathBuf::from(format!("{}.part", out_path.display()));
    let mut w = std::io::BufWriter::new(std::fs::File::create(&tmp)?);

    println!("Fetching CubeCobra Elo ratings ...");
    let mut page = 0usize;
    let mut total: Option<usize> = None;
    let mut written = 0usize;
    loop {
        if max_pages.is_some_and(|mp| page >= mp) {
            break;
        }
        let url = format!(
            "https://cubecobra.com/tool/api/topcards?f=cmc%3E%3D0&s=Elo&d=descending&p={page}"
        );
        let body = curl_text(&url).with_context(|| format!("cubecobra page {page}"))?;
        let v: serde_json::Value =
            serde_json::from_str(&body).with_context(|| format!("parsing cubecobra page {page}"))?;

        if total.is_none() {
            total = v["numResults"].as_u64().map(|n| n as usize);
            if let Some(t) = total {
                println!("  {t} cards (~{} pages)", t.div_ceil(CUBE_PAGE));
            }
        }
        let Some(data) = v["data"].as_array() else { break };
        if data.is_empty() {
            break;
        }
        for c in data {
            if c["isToken"].as_bool().unwrap_or(false) || c["isExtra"].as_bool().unwrap_or(false) {
                continue;
            }
            let rec = serde_json::json!({
                "name_lower": c["name_lower"],
                "oracle_id": c["oracle_id"],
                "elo": c["elo"],
                "cube_count": c["cubeCount"],
                "pick_count": c["pickCount"],
                "popularity": c["popularity"],
            });
            writeln!(w, "{}", serde_json::to_string(&rec)?)?;
            written += 1;
        }

        page += 1;
        if total.is_some_and(|t| page * CUBE_PAGE >= t) {
            break;
        }
        if page.is_multiple_of(25) {
            println!("  page {page} ({written} cards) ...");
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

    w.flush()?;
    drop(w);
    std::fs::rename(&tmp, &out_path)?;
    println!("  wrote {written} Elo records -> {}", out_path.display());
    Ok(())
}

fn inflate(gz: &Path, out: &Path) -> Result<()> {
    use flate2::read::GzDecoder;

    let f = std::fs::File::open(gz)?;
    let mut dec = GzDecoder::new(f);
    let tmp = PathBuf::from(format!("{}.part", out.display()));
    let mut w = std::fs::File::create(&tmp)?;
    std::io::copy(&mut dec, &mut w).context("gzip decompress")?;
    std::fs::rename(&tmp, out)?;
    Ok(())
}

fn curl_to_file(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-L", "--fail", "--silent", "--show-error", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .context("failed to spawn curl (is it installed / on PATH?)")?;
    if !status.success() {
        bail!("curl failed for {url}");
    }
    Ok(())
}

fn curl_text(url: &str) -> Result<String> {
    let out = Command::new("curl")
        .args(["-L", "--fail", "--silent", "--show-error", url])
        .output()
        .context("failed to spawn curl")?;
    if !out.status.success() {
        bail!("curl failed for {url}");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
