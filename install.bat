@echo off

cargo build --release
mkdir %LOCALAPPDATA%\.codesearch\
copy target\release\codesearch.exe %LOCALAPPDATA%\.codesearch\

echo Setup complete. To use codesearch from the command line, add %LOCALAPPDATA%\.codesearch to the system PATH.
