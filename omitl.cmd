@echo off
:: Omitl dev CLI for Windows CMD
:: Usage: omitl <command> [args]

if "%1"==""       goto help
if "%1"=="help"   goto help
if "%1"=="--help" goto help
if "%1"=="-h"     goto help
if "%1"=="run"    goto run
if "%1"=="build"  goto build
if "%1"=="check"  goto check
if "%1"=="test"   goto test
if "%1"=="fmt"    goto fmt
if "%1"=="lint"   goto lint
if "%1"=="example" goto example

echo Unknown command: %1
echo Run "omitl help" for available commands.
exit /b 1

:run
shift
cargo run -- %1 %2 %3 %4 %5 %6 %7 %8 %9
goto end

:build
cargo build --release
goto end

:check
cargo check
goto end

:test
shift
cargo test %1 %2 %3 %4 %5 %6 %7 %8 %9
goto end

:fmt
cargo fmt
goto end

:lint
cargo clippy -- -D warnings
goto end

:example
cargo run -- generate --input examples/api_contract.json --brand examples/brand.json --format pdf --output %TEMP%\omitl_example.pdf
echo Output: %TEMP%\omitl_example.pdf
goto end

:help
echo Omitl - API contract documentation generator
echo.
echo Development commands:
echo   omitl run [args]   Compile and run  (cargo run -- [args])
echo   omitl build        Release build    (cargo build --release)
echo   omitl check        Type-check only  (cargo check)
echo   omitl test         Run tests        (cargo test)
echo   omitl fmt          Format code      (cargo fmt)
echo   omitl lint         Clippy linter    (cargo clippy)
echo   omitl example      Generate sample PDF from examples/
goto end

:end
