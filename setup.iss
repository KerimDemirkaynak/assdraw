; Inno Setup script for ASSDraw-rs (Rust port of ASSDraw3)
; Build the release binary first, then compile this script with Inno Setup.
; Expected layout after `cargo build --release`:
;   target\release\assdraw-rs.exe
; Optional: place locale / help / docs next to the exe or under a package\ tree.

[Setup]
AppName=ASSDraw-rs
AppId=ASSDraw-rs
AppVersion=0.3.0
AppPublisher=ASSDraw-rs
DefaultDirName={autopf}\ASSDraw-rs
DefaultGroupName=ASSDraw-rs
OutputDir=installer
OutputBaseFilename=ASSDraw-rs-Windows-x64-Setup
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\assdraw-rs.exe
Compression=lzma
SolidCompression=yes
WizardStyle=modern
SetupIconFile=assets\assdraw.ico
WizardImageFile=assets\welcome-large.bmp
WizardSmallImageFile=assets\assdraw.bmp
ShowLanguageDialog=yes
LicenseFile=LICENSE

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "tr"; MessagesFile: "compiler:Languages\Turkish.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "ko"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "zh"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main binary (adjust path if you stage under package\)
Source: "target\release\assdraw-rs.exe"; DestDir: "{app}"; Flags: ignoreversion
; Optional assets / docs shipped with the app
Source: "assets\assdraw.ico"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "assets\assdraw.png"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\ASSDraw-rs"; Filename: "{app}\assdraw-rs.exe"; WorkingDir: "{app}"; IconFilename: "{app}\assdraw-rs.exe"
Name: "{autodesktop}\ASSDraw-rs"; Filename: "{app}\assdraw-rs.exe"; WorkingDir: "{app}"; IconFilename: "{app}\assdraw-rs.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\assdraw-rs.exe"; WorkingDir: "{app}"; Description: "{cm:LaunchProgram,ASSDraw-rs}"; Flags: nowait postinstall skipifsilent
