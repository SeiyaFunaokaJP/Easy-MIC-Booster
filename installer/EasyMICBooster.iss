; Easy MIC Booster installer (Inno Setup 6)
;
; Built by build\build_release.bat from the published self-contained output,
; so the installer and the portable zip contain the same files:
;   ISCC /DAppVersion=<ver> /DArch=x64|x86 /DPkgDir=<dir> /DOutputDir=<dir> EasyMICBooster.iss

#ifndef AppVersion
  #error AppVersion is not defined
#endif
#ifndef Arch
  #error Arch is not defined (x64 or x86)
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
; x64 and x86 share it, so installing one replaces the other.
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
#endif
MinVersion=10.0
; Same name as the app's single-instance mutex (App.xaml.cs): setup and the
; uninstaller ask to close a running Easy MIC Booster first
AppMutex=Global\EasyMICBooster_UniqueInstance_Mutex
CloseApplications=yes
SetupIconFile=..\src\Resources\App.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
ShowLanguageDialog=auto
LicenseFile=..\LICENSE
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

[Files]
Source: "{#PkgDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

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

{ "Start with Windows" writes HKCU\...\Run\EasyMICBooster; remove it when it
  points at this installation. The config folder is kept. }
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
