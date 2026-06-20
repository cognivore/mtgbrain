//! mtgbrain — download MTGJSON card data into a queryable SQLite database that
//! an LLM agent can drive with diverse SQL / full-text queries.

mod build;
mod download;
mod edit;
mod model;
mod query;
mod render;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "mtgbrain",
    version,
    about = "MTGJSON cards in a queryable SQLite DB for LLM-driven card search"
)]
struct Cli {
    /// Directory for downloaded JSON and the built SQLite DB.
    #[arg(long, env = "MTGBRAIN_DATA", default_value = "data", global = true)]
    data_dir: PathBuf,

    /// Path to the SQLite DB (default: <data-dir>/mtg.sqlite).
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download MTGJSON data into the data dir.
    Download {
        /// Also fetch AllPrintings (large).
        #[arg(long)]
        all: bool,
        /// Re-download even if present.
        #[arg(long)]
        force: bool,
    },
    /// Fetch CubeCobra card Elo ratings (cube-draft signal) into the data dir.
    Cubecobra {
        /// Stop after N pages (96 cards each); for testing.
        #[arg(long)]
        max_pages: Option<usize>,
    },
    /// Build the SQLite DB from downloaded JSON.
    Build {
        /// Skip rulings tables (smaller/faster).
        #[arg(long)]
        no_rulings: bool,
    },
    /// Download MTGJSON + CubeCobra Elo + build, in one step.
    Setup {
        /// Re-download even if present.
        #[arg(long)]
        force: bool,
        /// Skip the CubeCobra Elo fetch (faster).
        #[arg(long)]
        no_cube: bool,
    },
    /// Print the schema and a query cheatsheet (start here).
    Schema,
    /// Run a single read-only SELECT/WITH query.
    Sql {
        /// The SQL to run.
        query: String,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Full-text search over name/type/oracle text (ranked best-match first).
    Search {
        /// FTS5 query, e.g. 'counter NOT proliferate' or '"flip a coin"'.
        query: String,
        /// Extra WHERE clause; reference columns as c.<col> or bare.
        #[arg(long = "where")]
        where_clause: Option<String>,
        /// ORDER BY expression (default: rank = best match).
        #[arg(long, default_value = "rank")]
        order: String,
        /// Treat the query as one exact phrase.
        #[arg(long)]
        phrase: bool,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Show all faces + rulings for one card (by exact or partial name).
    Card {
        name: String,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Cube editor: seed an editor DB from a card list, then serve a local review UI.
    Edit {
        #[command(subcommand)]
        cmd: EditCmd,
    },
    /// Render old-frame MPC-ready PNGs from the editor DB (overrides applied).
    Render {
        #[command(subcommand)]
        cmd: RenderCmd,
    },
}

#[derive(Args, Clone)]
pub struct RenderCommon {
    /// Editor DB to read cards + overrides from (read-only).
    #[arg(long = "editor-db")]
    editor_db: Option<PathBuf>,
    /// Directory holding downloaded frame/font assets.
    #[arg(long, default_value = "assets")]
    assets: PathBuf,
    /// Output/cache directory for rendered cards + downloaded art.
    #[arg(long, default_value = "render-cache")]
    cache: PathBuf,
    /// Path to the Chrome/Chromium binary used for headless screenshots.
    #[arg(long, env = "MTGBRAIN_CHROME", default_value = render::CHROME_DEFAULT)]
    chrome: String,
    /// MPC-Autofill community backend URL for high-DPI art (else Scryfall art_crop).
    #[arg(long)]
    art_backend: Option<String>,
    /// Also render the foil (falling-star) variant.
    #[arg(long)]
    foil: bool,
    /// Re-render even if the cached hash file already exists.
    #[arg(long)]
    force: bool,
}

#[derive(Subcommand)]
enum RenderCmd {
    /// Download cardconjurer frame art, old fonts, foil overlay + mana font into ./assets.
    Assets {
        /// Re-download even if present.
        #[arg(long)]
        force: bool,
        /// Assets directory.
        #[arg(long, default_value = "assets")]
        dir: PathBuf,
    },
    /// Render a single card by editor-DB id.
    Card {
        id: i64,
        #[command(flatten)]
        common: RenderCommon,
    },
    /// Render every card in the editor DB.
    All {
        #[command(flatten)]
        common: RenderCommon,
    },
}

#[derive(Subcommand)]
enum EditCmd {
    /// Build the editor DB from a plain card-name list (one per line).
    Seed {
        /// Card list (one name per line; blank/`#` lines ignored).
        #[arg(long, default_value = "projects/odyssey2026/cube360/cube_list.txt")]
        list: PathBuf,
        /// Output editor DB (default: <data-dir>/cube_editor.sqlite).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Overwrite an existing editor DB (discards saved decisions).
        #[arg(long)]
        force: bool,
    },
    /// Serve the review UI on a local port.
    Serve {
        /// Editor DB (default: <data-dir>/cube_editor.sqlite).
        #[arg(long = "editor-db")]
        editor_db: Option<PathBuf>,
        /// Port (deliberately high to avoid clashes).
        #[arg(long, default_value_t = 49737)]
        port: u16,
        /// Assets dir for old-frame render.
        #[arg(long, default_value = "assets")]
        assets: PathBuf,
        /// Render cache dir.
        #[arg(long, default_value = "render-cache")]
        cache: PathBuf,
        /// Chrome binary for headless render.
        #[arg(long, env = "MTGBRAIN_CHROME", default_value = render::CHROME_DEFAULT)]
        chrome: String,
        /// MPC-Autofill backend URL for high-DPI art (else Scryfall).
        #[arg(long)]
        art_backend: Option<String>,
    },
}

#[derive(Args, Clone)]
pub struct OutputArgs {
    /// Output format (or use the --json / --md / --csv shorthands).
    #[arg(short = 'f', long, value_enum, default_value_t = Format::Table)]
    pub format: Format,
    /// Shorthand for --format json.
    #[arg(long, conflicts_with = "format")]
    pub json: bool,
    /// Shorthand for --format md.
    #[arg(long, conflicts_with = "format")]
    pub md: bool,
    /// Shorthand for --format csv.
    #[arg(long, conflicts_with = "format")]
    pub csv: bool,
    /// Max rows to print (sql/card). 0 = unlimited.
    #[arg(long, default_value_t = 40)]
    pub limit: usize,
    /// Do not truncate long text.
    #[arg(long)]
    pub full: bool,
    /// Max cell width before truncation.
    #[arg(long, default_value_t = 70)]
    pub width: usize,
    /// Comma-separated columns to display.
    #[arg(long)]
    pub cols: Option<String>,
}

impl OutputArgs {
    /// Resolve the effective format, honoring the --json/--md/--csv shorthands.
    pub fn fmt(&self) -> Format {
        if self.json {
            Format::Json
        } else if self.md {
            Format::Md
        } else if self.csv {
            Format::Csv
        } else {
            self.format
        }
    }
}

#[derive(Copy, Clone, ValueEnum)]
pub enum Format {
    Table,
    Json,
    Md,
    Csv,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = cli
        .db
        .clone()
        .unwrap_or_else(|| cli.data_dir.join("mtg.sqlite"));

    match &cli.cmd {
        Cmd::Download { all, force } => download::download(&cli.data_dir, *all, *force),
        Cmd::Cubecobra { max_pages } => download::cubecobra(&cli.data_dir, *max_pages),
        Cmd::Build { no_rulings } => build::build(&cli.data_dir, &db, !*no_rulings),
        Cmd::Setup { force, no_cube } => {
            download::download(&cli.data_dir, false, *force)?;
            if !*no_cube {
                download::cubecobra(&cli.data_dir, None)?;
            }
            build::build(&cli.data_dir, &db, true)
        }
        Cmd::Schema => query::schema(&db),
        Cmd::Sql { query, out } => query::run_sql(&db, query, out),
        Cmd::Search {
            query,
            where_clause,
            order,
            phrase,
            out,
        } => query::search(&db, query, where_clause.as_deref(), order, *phrase, out),
        Cmd::Card { name, out } => query::card(&db, name, out),
        Cmd::Edit { cmd } => match cmd {
            EditCmd::Seed { list, out, force } => {
                let out = out
                    .clone()
                    .unwrap_or_else(|| edit::default_editor_db(&cli.data_dir));
                edit::seed(&db, list, &out, *force)
            }
            EditCmd::Serve {
                editor_db,
                port,
                assets,
                cache,
                chrome,
                art_backend,
            } => {
                let edb = editor_db
                    .clone()
                    .unwrap_or_else(|| edit::default_editor_db(&cli.data_dir));
                let rc = edit::RenderCfg {
                    assets: assets.clone(),
                    cache: cache.clone(),
                    chrome: chrome.clone(),
                    backend: art_backend.clone(),
                    source_db: db.clone(),
                };
                edit::serve(&edb, *port, &rc)
            }
        },
        Cmd::Render { cmd } => match cmd {
            RenderCmd::Assets { force, dir } => render::assets(dir, *force),
            RenderCmd::Card { id, common } => {
                let edb = common
                    .editor_db
                    .clone()
                    .unwrap_or_else(|| edit::default_editor_db(&cli.data_dir));
                let out = render::render_one(
                    &edb, &common.assets, &common.cache, &common.chrome, *id, common.foil,
                    common.force, common.art_backend.as_deref(),
                )?;
                println!("{}", out.display());
                Ok(())
            }
            RenderCmd::All { common } => {
                let edb = common
                    .editor_db
                    .clone()
                    .unwrap_or_else(|| edit::default_editor_db(&cli.data_dir));
                render::render_all(
                    &edb, &common.assets, &common.cache, &common.chrome, common.foil,
                    common.force, common.art_backend.as_deref(),
                )
            }
        },
    }
}
