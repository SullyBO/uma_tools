@echo off
set APP_ENV=%1
if "%APP_ENV%"=="prod" (
    for /f "usebackq tokens=1,* delims==" %%a in (".env.prod") do (
        if /i "%%a"=="DATABASE_URL" set DATABASE_URL=%%b
    )
) else (
    for /f "usebackq tokens=1,* delims==" %%a in (".env") do (
        if /i "%%a"=="DATABASE_URL" set DATABASE_URL=%%b
    )
)
cargo sqlx migrate run