@echo off
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
if exist "C:\Program Files\CMake\bin\cmake.exe" set "PATH=C:\Program Files\CMake\bin;%PATH%"
if exist "C:\Program Files\LLVM\bin\libclang.dll" set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
npx --no-install tauri %*
