@echo off
setlocal

set "ROOT_DIR=%~dp0"
pushd "%ROOT_DIR%"

wasm-pack build --target web
set "RC=%ERRORLEVEL%"
if %RC% NEQ 0 (
    echo wasm-pack build failed with exit code %RC%.
    popd
    endlocal
    exit /b %RC%
)

call "%ROOT_DIR%sync_docs.bat"
set "SYNC_RC=%ERRORLEVEL%"

popd
endlocal
exit /b %SYNC_RC%
