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
ShowLanguageDialog=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "bg"; MessagesFile: "compiler:Languages\Bulgarian.isl"
Name: "ca"; MessagesFile: "compiler:Languages\Catalan.isl"
Name: "cz"; MessagesFile: "compiler:Languages\Czech.isl"
Name: "da"; MessagesFile: "compiler:Languages\Danish.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "fi"; MessagesFile: "compiler:Languages\Finnish.isl"
Name: "fr_FR"; MessagesFile: "compiler:Languages\French.isl"
Name: "hu"; MessagesFile: "compiler:Languages\Hungarian.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "ko"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "nl"; MessagesFile: "compiler:Languages\Dutch.isl"
Name: "pl"; MessagesFile: "compiler:Languages\Polish.isl"
Name: "pt_BR"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "pt_PT"; MessagesFile: "compiler:Languages\Portuguese.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "tr"; MessagesFile: "compiler:Languages\Turkish.isl"
Name: "uk_UA"; MessagesFile: "compiler:Languages\Ukrainian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "docs\ASSDraw3.chm"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "build-dir\Release\locale\*"; DestDir: "{app}\bin\locale"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "build-dir\Release\assdraw3.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "build-dir\Release\*.dll"; DestDir: "{app}\bin"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\ASSDraw3"; Filename: "{app}\bin\assdraw3.exe"
Name: "{autodesktop}\ASSDraw3"; Filename: "{app}\bin\assdraw3.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\assdraw3.exe"; Description: "{cm:LaunchProgram,ASSDraw3}"; Flags: nowait postinstall skipifsilent
