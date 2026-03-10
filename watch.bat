@echo off
setlocal

pushd "%~dp0"

set "SYNC_SCRIPT=%CD%\sync_docs.bat"
set "BUILD_AND_SYNC=%CD%\build_and_sync.bat"

echo Running initial build and docs sync...
call "%BUILD_AND_SYNC%"
if errorlevel 1 (
    echo Initial build failed. Aborting watch setup.
    goto :END
)

:: Watch and recompile SCSS in the background
start "SASS Watch" sass --watch static/style.scss:static/style.css

:: Watch and rebuild the Yew project, syncing docs afterward
start "Cargo Watch" cargo watch -w src -s "build_and_sync.bat"

:: Watch for asset and static changes to keep docs in sync
start "Docs Sync" cargo watch -w static -w icons -w assets -w index.html -w manifest.json -w service-worker.js -s "sync_docs.bat"

:: Serve the files and enable live reload, watching for JSON changes too
start "Browser Sync" browser-sync start --server --files "static/*.css, pkg/*, assets/*.json" --startPath index.html

echo.
echo Watchers started. Docs output will stay in sync while this script is running.

:END
popd
endlocal
