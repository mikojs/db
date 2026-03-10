# DB

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

Add this to your nvim config:

```lua
local db_sqls_command = io.popen("db sqls")
local connections = vim.json.decode(db_sqls_command:read("*a"))

db_sqls_command:close()
vim.lsp.config("sqls", {
  capabilities = capabilities,
  settings = {
    sqls = {
      connections = connections,
    },
  },
})
vim.lsp.enable("sqls")
```

## Shell Completion

Generate shell completion scripts:

```bash
db --generate <bash|zsh|fish> > shell init script
```
