#!/bin/zsh
# Usage:
#   ./migrate.sh        # defaults to dev
#   ./migrate.sh prod   # runs against prod

APP_ENV=${1:-dev}
if [ "$APP_ENV" = "prod" ]; then
    export DATABASE_URL=$(grep DATABASE_URL .env.prod | cut -d '=' -f2-)
else
    export DATABASE_URL=$(grep DATABASE_URL .env | cut -d '=' -f2-)
fi
sqlx migrate run --database-url "$DATABASE_URL"