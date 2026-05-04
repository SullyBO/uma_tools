@echo off
set APP_ENV=%1
if "%APP_ENV%"=="prod" (
    for /f "tokens=2 delims==" %%a in ('findstr /i "DATABASE_URL" .env.prod') do set DATABASE_URL=%%a
) else (
    for /f "tokens=2 delims==" %%a in ('findstr /i "DATABASE_URL" .env') do set DATABASE_URL=%%a
)
cargo sqlx migrate run