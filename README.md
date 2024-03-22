# Makishima - Backend

Makishima's backend is written in rust, utilizing actix-web and SeaORM.

## Building

Before running, environment variables are required to be set up. They are
listed below along with their purpose.

```
// Discord
MAKISHIMA_ID       - The client ID obtained from Discord.
MAKISHIMA_SECRET   - The client secret obtained from Discord.
MAKISHIMA_REDIRECT - The redirect used for Discord OAuth.
MAKISHIMA_SIGKEY   - The key used for signing JWT tokens.

// Database
DB_URI - The database URI.

// Development
RUST_LOG - The log level to be used during runtime.
```

After setting up the environment variables, compile and run the backend
accordingly.
