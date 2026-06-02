@echo off
set APP_ENV=prod
for /f "usebackq tokens=1,* delims==" %%a in (".env.prod") do (
    if /i "%%a"=="DATABASE_URL" set DATABASE_URL=%%b
)
cargo run -p uma_cli sync all