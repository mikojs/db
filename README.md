# db

Parse database information from environment variables.

## Installation

```bash
cargo install --path .
```

## Usage

Set environment variables for your databases:

```bash
export DB_<DB_NAME>_URL=<database_url>
export DB_<DB_NAME>_TYPE=<postgresql|sqlite3>
export DB_<DB_NAME>_DESCRIPTION=<description>

# Example
export DB_PG_URL="postgresql://postgres:postgres@localhost/postgres"
export DB_PG_TYPE="postgresql"
export DB_PG_DESCRIPTION="Default database"

export DB_SQLITE_URL="file:foo.db"
export DB_SQLITE_TYPE="sqlite3"
```

### Show database URL

```bash
db show <db_name>
```

### Generate sqls config

```bash
db sqls
```

Output:

```json
[
  {
    "driver": "postgresql",
    "dataSourceName": "postgresql://postgres:postgres@localhost/postgres"
  }
]
```

## Shell Completion

Generate shell completion scripts:

```bash
coder --generate <bash|zsh|fish> > shell init script
```
