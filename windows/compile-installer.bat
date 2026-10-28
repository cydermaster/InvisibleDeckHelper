@echo off
rem Compile the Inno Setup installer. Called on the Windows box so that no
rem quoting has to survive an ssh -> cmd hand-off.
rem
rem Usage: compile-installer.bat <version> <bundle-dir>
setlocal enabledelayedexpansion
set "ISCC="
if exist "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" set "ISCC=%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe"
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles%\Inno Setup 6\ISCC.exe"

if "%ISCC%"=="" (
  echo ERROR: ISCC.exe not found
  exit /b 2
)

echo Using: %ISCC%
echo Version: %~1
echo Bundle: %~2

"%ISCC%" /DMyAppVersion=%~1 /DBundleDir=%~2 "%~dp0installer.iss"
set RC=%ERRORLEVEL%
echo ISCC exit code: %RC%
exit /b %RC%
