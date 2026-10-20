; Easy MIC Booster installer (Inno Setup 6)
;
; Built by build\build_release.ps1 (and .github\workflows\release.yml) from the
; staged release folder, so the installer and the portable zip contain the same
; files: EasyMICBooster.exe (a single self-contained binary with the languages
; embedded), LICENSE-MIT, LICENSE-APACHE, THIRD-PARTY-NOTICES.txt and README.md.
;   ISCC /DAppVersion=<ver> /DArch=x64|x86|arm64 /DPkgDir=<dir> /DOutputDir=<dir> [/DLicensePage=<file>] EasyMICBooster.iss
; LicensePage is the text for the license page (build_release.ps1 combines
; both licenses); without it the page shows LICENSE-MIT.

#ifndef AppVersion
  #error AppVersion is not defined
#endif
#ifndef Arch
  #error Arch is not defined (x64, x86 or arm64)
#endif
#if Arch != "x64" && Arch != "x86" && Arch != "arm64"
  #error Arch must be x64, x86 or arm64
#endif
#ifndef PkgDir
  #error PkgDir is not defined
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

#define AppName "Easy MIC Booster"
#define AppExe "EasyMICBooster.exe"
#define RepoUrl "https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster"

[Setup]
; Never change AppId: it ties upgrades and the uninstaller to earlier installs.
; x64, x86 and arm64 share it, so installing one replaces the other.
AppId={{70F2979B-CA0E-4236-841B-164616251CFC}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Seiya Funaoka
AppPublisherURL={#RepoUrl}
AppSupportURL={#RepoUrl}/issues
AppUpdatesURL={#RepoUrl}/releases
VersionInfoVersion={#AppVersion}
; Per-user by default (no administrator rights); the dialog offers all users
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={autopf}\EasyMICBooster
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
#if Arch == "x64"
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#elif Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#endif
MinVersion=10.0
; Same name as the app's single-instance mutex (unchanged since 1.x): setup and
; the uninstaller ask to close a running Easy MIC Booster first
AppMutex=Global\EasyMICBooster_UniqueInstance_Mutex
CloseApplications=yes
SetupIconFile=..\assets\App.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
ShowLanguageDialog=auto
#ifdef LicensePage
LicenseFile={#LicensePage}
#else
LicenseFile=..\LICENSE-MIT
#endif
Compression=lzma2/max
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=EasyMICBooster-{#Arch}-v{#AppVersion}-setup

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "ko"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[InstallDelete]
; Upgrading over 1.x (.NET self-contained build): remove its runtime files,
; which the single Rust executable no longer needs. Only files inside the
; application folder are touched; settings in %APPDATA% are not.
Type: files; Name: "{app}\*.dll"
Type: files; Name: "{app}\*.pdb"
Type: files; Name: "{app}\*.deps.json"
Type: files; Name: "{app}\*.runtimeconfig.json"
Type: files; Name: "{app}\dotnet-THIRD-PARTY-NOTICES.txt"
Type: filesandordirs; Name: "{app}\Lang"
Type: filesandordirs; Name: "{app}\runtimes"

[Files]
Source: "{#PkgDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PkgDir}\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PkgDir}\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PkgDir}\THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PkgDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[Code]
const
  RunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';
  ApprovedKey = 'Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run';

{ On a first install, start Easy MIC Booster in the language chosen for setup.
  The settings, presets and device profiles live in %APPDATA%\EasyMICBooster
  (shared with the portable version) and are left alone if they already exist.
  Chinese has no bundled setup translation; the app picks it from the Windows
  UI language on first run instead. }
procedure CurStepChanged(CurStep: TSetupStep);
var
  Dir: String;
begin
  if CurStep <> ssPostInstall then Exit;
  Dir := ExpandConstant('{userappdata}\EasyMICBooster');
  if FileExists(Dir + '\config.ini') then Exit;
  { $04 = LANG_CHINESE (primary language of the Windows UI LANGID) }
  if (ActiveLanguage = 'en') and ((GetUILanguage and $3FF) = $04) then Exit;
  ForceDirectories(Dir);
  SaveStringToFile(Dir + '\config.ini',
    '[Settings]' + #13#10 + 'Language=' + ActiveLanguage + #13#10, False);
end;

{ "Start with Windows" writes HKCU\...\Run\EasyMICBooster as
  "<app>\EasyMICBooster.exe" --minimized; remove it when it points at this
  installation. The config folder is kept. }
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Cmd: String;
begin
  if CurUninstallStep <> usPostUninstall then Exit;
  if RegQueryStringValue(HKCU, RunKey, 'EasyMICBooster', Cmd) and
     (Pos(Lowercase(ExpandConstant('{app}\{#AppExe}')), Lowercase(Cmd)) > 0) then
  begin
    RegDeleteValue(HKCU, RunKey, 'EasyMICBooster');
    RegDeleteValue(HKCU, ApprovedKey, 'EasyMICBooster');
  end;
end;
