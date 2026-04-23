# uma_tools

# uma

A data pipeline and API backend for an Umamusume: Pretty Derby discord bot.

Scrapes the [Umamusume wiki](https://umamusume.wiki) and [GameTora](https://gametora.com) on a monthly cadence, seeds a PostgreSQL database, and exposes the data for consumption by a Discord bot (separate project).

## Workspace Structure

```
uma/
├── uma-core/       # Shared domain models and types
├── uma-scraper/    # Web scrapers (wiki + GameTora)
├── uma-db/         # Repository layer (SQLx + PostgreSQL)
├── uma-api/        # HTTP API server
└── uma-cli/        # CLI entrypoint
```

## CLI

```
uma sync skills       # Scrape and upsert skill data
uma sync characters   # Scrape and upsert character data
uma sync all          # Both
```

## Tech Stack

- **Language:** Rust
- **Database:** PostgreSQL via [SQLx](https://github.com/launchbadge/sqlx)
- **HTTP client:** reqwest
- **HTML parsing:** scraper
- **CLI:** clap
- **Logging:** log + env_logger

## Setup

1. Clone the repo
2. Copy `.env.example` to `.env` and fill in your `DATABASE_URL`
3. Run migrations: `uma db migrate`
4. Sync data: `uma sync all`

## Status

It's a work in progress. Discord bot frontend is a planned separate project.

## License

MIT, do whatever you want with it.