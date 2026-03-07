@echo off

echo Building WASM...

where wasm-pack >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo wasm-pack not found. Please install:
    echo   cargo install wasm-pack
    exit /b 1
)

wasm-pack build --target web --out-dir examples/pkg --features wasm

echo.
echo Build complete!
echo.
echo To run:
echo   cd examples
echo   npx serve
