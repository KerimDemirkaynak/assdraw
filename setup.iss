[Setup]
AppName=ASSDraw3
AppVersion=3.0
DefaultDirName={autopf}\ASSDraw3
DefaultGroupName=ASSDraw3
OutputDir=installer
OutputBaseFilename=ASSDraw3_Setup
Compression=lzma
SolidCompression=yes
WizardImageFile=docs\resources\welcome-large.bmp
WizardSmallImageFile=docs\resources\assdraw.bmp
SetupIconFile=docs\resources\assdraw.ico

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "build-dir\Release\assdraw3.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "build-dir\Release\*.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\ASSDraw3"; Filename: "{app}\assdraw3.exe"
Name: "{autodesktop}\ASSDraw3"; Filename: "{app}\assdraw3.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\assdraw3.exe"; Description: "{cm:LaunchProgram,ASSDraw3}"; Flags: nowait postinstall skipifsilent
