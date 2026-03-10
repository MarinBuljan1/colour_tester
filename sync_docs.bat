@echo off
setlocal

set "ROOT_DIR=%~dp0"
pushd "%ROOT_DIR%"

set "DOCS_DIR=%CD%\docs"
if not exist "%DOCS_DIR%" (
    mkdir "%DOCS_DIR%"
)

call :CopyFile index.html
call :CopyFile manifest.json
if exist service-worker.js (
    call :CopyFile service-worker.js
)

call :SyncDir static "%DOCS_DIR%\static"
call :SyncDir icons "%DOCS_DIR%\icons"
call :SyncDir assets "%DOCS_DIR%\assets"
if exist pkg (
    call :SyncDir pkg "%DOCS_DIR%\pkg"
)

popd
endlocal
exit /b 0

:CopyFile
set "SOURCE=%~1"
if not exist "%SOURCE%" (
    echo Skipping missing file "%SOURCE%".
    exit /b 0
)
copy /Y "%SOURCE%" "%DOCS_DIR%\%~1" >nul
exit /b 0

:SyncDir
set "SOURCE_DIR=%~1"
set "TARGET_DIR=%~2"

if not exist "%SOURCE_DIR%" (
    echo Skipping missing source "%SOURCE_DIR%".
    exit /b 0
)

robocopy "%SOURCE_DIR%" "%TARGET_DIR%" /MIR >nul
set "RC=%ERRORLEVEL%"
if %RC% GEQ 8 (
    echo Failed to copy "%SOURCE_DIR%" to "%TARGET_DIR%".
    exit /b %RC%
)
exit /b 0
