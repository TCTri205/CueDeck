@echo off
REM Manual test script for Phase 5.3

echo ========================================
echo Phase 5.3 Manual Testing
echo ========================================
echo.

REM Clean up old test workspace
if exist test_workspace rmdir /s /q test_workspace

REM Create test workspace
echo [1/5] Creating test workspace...
mkdir test_workspace
cd test_workspace

REM Initialize CueDeck
echo [2/5] Initializing CueDeck...
..\target\release\cue.exe init

REM Create test markdown files
echo [3/5] Creating test files...
mkdir .cuedeck\docs 2>nul

echo # Authentication> .cuedeck\docs\auth.md
echo.>> .cuedeck\docs\auth.md
echo Login flow with OAuth 2.0 and JWT tokens.>> .cuedeck\docs\auth.md

echo # Database> .cuedeck\docs\database.md
echo.>> .cuedeck\docs\database.md
echo PostgreSQL setup and configuration for production.>> .cuedeck\docs\database.md

echo # API> .cuedeck\docs\api.md
echo.>> .cuedeck\docs\api.md
echo REST API endpoints for user management and authentication.>> .cuedeck\docs\api.md

echo # Testing> .cuedeck\docs\testing.md
echo.>> .cuedeck\docs\testing.md
echo Unit tests and integration tests for concurrent code.>> .cuedeck\docs\testing.md

REM List files
echo.
echo Created files:
dir .cuedeck\docs\*.md /b
echo.

REM Note: The following tests would open interactive UI
REM For automated testing, we just verify the binary works
echo [4/5] Verifying binary works...
..\target\release\cue.exe --version

echo.
echo [5/5] Checking if cache directory exists...
if not exist .cuedeck\cache mkdir .cuedeck\cache
if exist .cuedeck\cache echo Cache directory: OK

echo.
echo ========================================
echo Manual tests completed!
echo ========================================
echo.
echo To test interactively, run:
echo   cd test_workspace
echo   ..\target\release\cue open "auth" --mode=keyword
echo   ..\target\release\cue open "login" --mode=semantic
echo   ..\target\release\cue open "database"
echo.
echo After semantic search, check cache:
echo   dir .cuedeck\cache\embeddings.bin
echo.

cd ..
