use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ppl", about = "Personal CRM for colleagues and acquaintances")]
pub struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Path to database file
    #[arg(long, global = true)]
    pub db: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a new person
    Add {
        /// Full name
        name: String,
        #[arg(long)]
        nickname: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        company: Option<String>,
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        department: Option<String>,
        #[arg(long)]
        job_title: Option<String>,
        #[arg(long)]
        birthday: Option<String>,
        #[arg(long)]
        employment_date: Option<String>,
    },
    /// Edit an existing person
    Edit {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
        #[arg(long)]
        nickname: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        company: Option<String>,
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        department: Option<String>,
        #[arg(long)]
        job_title: Option<String>,
        #[arg(long)]
        birthday: Option<String>,
        #[arg(long)]
        employment_date: Option<String>,
        #[arg(long)]
        set_name: Option<String>,
    },
    /// Show details for a person
    Show {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
    },
    /// List all people
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Filter by company
        #[arg(long)]
        company: Option<String>,
        /// Sort order: name, created, company
        #[arg(long, default_value = "name")]
        sort: String,
    },
    /// Remove a person
    Rm {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
    },
    /// Add a note to a person
    Note {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
        /// Note text
        text: Option<String>,
    },
    /// Add a tag to a person
    Tag {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
        /// Tag to add
        #[arg(long)]
        tag: Option<String>,
    },
    /// Remove a tag from a person
    Untag {
        /// Name to search for (omit for fuzzy picker)
        name: Option<String>,
        /// Tag to remove
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show upcoming date events
    Dates {
        /// Range: today, tomorrow, this-week, this-month, this-year, next-7d, next-30d, next-90d, last-7d, last-30d
        range: String,
    },
    /// Search across all fields, notes, and tags
    Search {
        /// Search query
        query: String,
    },
    /// Export all data to a JSON file
    Export {
        /// Output file path (defaults to stdout)
        path: Option<String>,
    },
    /// Import data from a JSON file
    Import {
        /// Input file path
        path: String,
    },
}
