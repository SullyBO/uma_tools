#!/bin/bash
APP_ENV=${1:-dev}
if [ "$APP_ENV" = "prod" ]; then
    export DATABASE_URL=$(grep -i DATABASE_URL .env.prod | cut -d '=' -f2)
else
    export DATABASE_URL=$(grep -i DATABASE_URL .env | cut -d '=' -f2)
fi
cargo sqlx migrate run