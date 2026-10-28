@echo off
rem Install/uninstall cycle test for the Inno Setup package.
rem Usage: test-install.bat <setup.exe> <install-dir>
setlocal
set "SETUP=%~1"
set "DIR=%~2"

echo === cleaning previous test dir ===
if exist "%DIR%" rd /s /q "%DIR%"

echo === install ===
"%SETUP%" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /NOICONS /DIR="%DIR%"
echo INSTALL_EXIT=%ERRORLEVEL%

echo === installed tree (top level) ===
dir /b "%DIR%"

echo === subfolders ===
dir /b /ad "%DIR%"

echo === payload file count ===
dir /b /s "%DIR%\python" 2>nul | find /c /v ""

echo === run the installed copy ===
pushd "%DIR%"
invisible-deck-helper.exe --version
invisible-deck-helper.exe --speech-info
invisible-deck-helper.exe --print --seed 42 --pair
popd

echo === uninstall (silent) ===
"%DIR%\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
echo UNINSTALL_EXIT=%ERRORLEVEL%
echo === test script finished ===
