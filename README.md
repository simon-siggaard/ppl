# ppl

A personal CRM for the command line. Keep track of colleagues, acquaintances, and the details that matter — names, companies, birthdays, notes, tags, and more.

Data is stored locally in a SQLite database. No accounts, no cloud, no sync.

## Usage

```
$ ppl --help
Personal CRM for colleagues and acquaintances

Usage: ppl [OPTIONS] <COMMAND>

Commands:
  add     Add a new person
  edit    Edit an existing person
  show    Show details for a person
  list    List all people
  rm      Remove a person
  note    Add a note to a person
  tag     Add a tag to a person
  untag   Remove a tag from a person
  dates   Show upcoming date events
  search  Search across all fields, notes, and tags
  export  Export all data to a JSON file
  import  Import data from a JSON file
  help    Print this message or the help of the given subcommand(s)

Options:
      --json      Output as JSON
      --no-color  Disable colored output
      --db <DB>   Path to database file
  -h, --help      Print help
```

## Install

```
cargo install --path .
```

## Quick start

```bash
# Add someone (interactive prompts for all fields)
ppl add "Jane Smith"

# Add someone with flags (skip the prompts)
ppl add "Jane Smith" --email jane@example.com --company Acme --job-title "Staff Engineer"

# View their details
ppl show "Jane"

# List everyone
ppl list
```

When a name argument is omitted, a fuzzy picker lets you select from existing entries. When a name matches multiple people, you're prompted to disambiguate.

## Person fields

name, nickname, email, phone, company, team, department, job title, birthday, employment date — plus freeform notes, tags, and custom fields.

## Filtering and sorting

```bash
# Filter by tag or company
ppl list --tag friend
ppl list --company Acme

# Sort by name (default), created, or company
ppl list --sort company
```

## Date queries

Track birthdays and employment anniversaries with date range queries:

```bash
ppl dates this-week
ppl dates next-30d
ppl dates this-month
```

Supported ranges: `today`, `tomorrow`, `this-week`, `this-month`, `this-year`, `next-7d`, `next-30d`, `next-90d`, `last-7d`, `last-30d`.

## Search

Full-text search across names, emails, companies, notes, tags, and custom fields — with fuzzy name matching as a fallback:

```bash
ppl search "acme"
ppl search "birthday party"
```

## Export and import

Move your data between machines with JSON export/import:

```bash
# Export to a file
ppl export ~/ppl-backup.json

# Export to stdout (for piping)
ppl export

# Import on another machine
ppl import ~/ppl-backup.json
```

## Global flags

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON (for scripting and piping) |
| `--no-color` | Disable colored output (also respects `NO_COLOR` env var) |
| `--db <path>` | Use a custom database file |

## Data storage

The database lives at:

- **Custom path:** `--db <path>`
- **`PPL_DIR` env var:** `$PPL_DIR/ppl.db`
- **Default:** `$XDG_DATA_HOME/ppl/ppl.db` (typically `~/.local/share/ppl/ppl.db`)

## License

MIT
