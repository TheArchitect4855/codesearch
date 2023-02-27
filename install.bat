@echo off

cargo build --release
mkdir %LOCALAPPDATA%\.codesearch\
copy target\release\codesearch.exe %LOCALAPPDATA%\.codesearch\
setx PATH "%PATH%;%LOCALAPPDATA%\.codesearch\"

echo Built codesearch, moved it to %LOCALAPPDATA%\.codesearch\ and added it to PATH.
