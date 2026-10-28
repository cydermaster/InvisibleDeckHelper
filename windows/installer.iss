; Inno Setup script for Invisible Deck Helper (Windows 10/11, 64-bit)
;
; The app is a portable folder: the exe plus python\ and voices\, which it needs
; in order to speak. The installer copies that whole folder, so it must be
; compiled against a bundle produced by build-portable-zip.ps1 / .sh — never
; against a bare exe.
;
; Compile:
;   ISCC.exe /DMyAppVersion=1.3.0 /DBundleDir=<bundle folder> installer.iss
;
; build-installer.ps1 wraps that call, reads the version from Cargo.toml and
; checks the bundle matches it.
;
; When a code-signing certificate exists, add e.g.:
;   SignTool=signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 $f
; ...or pass /Ssigntool="..." on the ISCC command line. Unsigned builds show
; SmartScreen's "Windows protected your PC" prompt on first run.

#ifndef MyAppVersion
  #error Pass /DMyAppVersion=<version> when compiling
#endif
#ifndef BundleDir
  #error Pass /DBundleDir=<path to the built portable Windows folder>
#endif

#define MyAppName "Invisible Deck Helper"
#define MyAppPublisher "Cydermaster"
#define MyAppURL "https://github.com/cydermaster/InvisibleDeckHelper"
#define MyAppExeName "invisible-deck-helper.exe"

[Setup]
; A stable AppId means upgrades replace the previous install instead of
; stacking a second copy in Add/Remove Programs. Do not change it.
AppId={{6F1B4C8E-2A3D-4E7B-9C51-8D2F4A7B1E63}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Setup
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}

; Per-user by default so there is no UAC prompt; the installer offers an
; all-users install if run with /ALLUSERS.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes

LicenseFile=..\LICENSE
SetupIconFile=invisible-deck-helper.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName} {#MyAppVersion}
OutputDir=..\dist
OutputBaseFilename=InvisibleDeckHelper-{#MyAppVersion}-windows-x86_64-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
; The bundle is ~240 MB; closing a running copy avoids a locked-file failure.
CloseApplications=yes
RestartApplications=no
AllowNoIcons=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; The whole bundle, subfolders included — python\ and voices\ are not optional.
Source: "{#BundleDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Inno Setup only removes files it installed, so anything a run may leave in
; the bundled folders would keep the install directory alive. Name them here.
Type: filesandordirs; Name: "{app}\python"
Type: filesandordirs; Name: "{app}\voices"
Type: dirifempty; Name: "{app}"

[Code]
// Belt and braces on top of [UninstallDelete]: wipe whatever is left of the
// application directory once the uninstall has finished. Inno will still be
// holding its own uninstaller stub at this point, so in practice a single
// unins000.exe (about 1 MB) can survive an uninstall along with its folder —
// everything the installer put there is gone, and Add/Remove Programs is clean.
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    DelTree(ExpandConstant('{app}'), True, True, True);
  end;
end;

