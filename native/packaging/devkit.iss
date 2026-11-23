#define MyAppName "DevKit"
#define MyAppVersion "0.3.0"
#define MyAppPublisher "DevKit"
#define MyAppExeName "devkit.exe"

[Setup]
AppId={{A9C1B7D2-4E6F-4A2B-9C3D-8E5F1A6D7B90}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={userappdata}\DevKit
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\target
OutputBaseFilename=DevKit-0.3.0-Setup
SetupIconFile=..\assets\devkit.ico
Compression=lzma2/max
SolidCompression=yes
PrivilegesRequired=lowest
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop icon"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "..\target\release\devkit.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch DevKit"; Flags: nowait postinstall skipifsilent
