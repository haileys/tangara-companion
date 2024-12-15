; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!

#define MyAppName "Tangara Companion"
#define MyAppPublisher "Hailey Somerville"
#define MyAppURL "https://github.com/haileys/tangara-companion"
#define MyAppExeName "tangara-companion.exe"
#define MyAppAssocName "Tangara Release Archive"
#define MyAppAssocExt ".tra"
#define MyAppAssocKey StringChange(MyAppAssocName, " ", "") + MyAppAssocExt

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{FDAA85D8-0554-4782-923C-93AF14626F05}
AppName={#MyAppName}
AppVersion={#AppVersion}
;AppVerName={#MyAppName} {#AppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
ChangesAssociations=yes
DisableProgramGroupPage=yes
LicenseFile={#ProjectDir}\COPYING
; Remove the following line to run in administrative install mode (install for all users.)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir={#DistDir}
OutputBaseFilename={#SetupExeName}
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#CargoTargetDir}\{#MyAppExeName}"; DestDir: "{app}\bin\"; Flags: ignoreversion

; direct dll dependencies:
Source: "{#GtkDir}\bin\adwaita-1-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\cairo-2.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\cairo-gobject-2.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\cairo-script-interpreter-2.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\epoxy-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\ffi-8.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\freetype-6.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\fribidi-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\gdk_pixbuf-2.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\gio-2.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\glib-2.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\gmodule-2.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\gobject-2.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\graphene-1.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\gtk-4-1.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\harfbuzz.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\iconv.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\intl.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\jpeg62.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\libexpat.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\libpng16.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\libxml2.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\pango-1.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\pangocairo-1.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\pangowin32-1.0-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\pcre2-8-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\pixman-1-0.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\rsvg-2-2.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\tiff.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion
Source: "{#GtkDir}\bin\zlib1.dll"; DestDir: "{app}\bin\"; Flags: ignoreversion

; rsvg dynamic loader for gdk-pixbuf
Source: "{#GtkDir}\lib\gdk-pixbuf-2.0\2.10.0\loaders.cache"; DestDir: "{app}\lib\gdk-pixbuf-2.0\2.10.0\"; Flags: ignoreversion
Source: "{#GtkDir}\lib\gdk-pixbuf-2.0\2.10.0\loaders\pixbufloader_svg.dll"; DestDir: "{app}\lib\gdk-pixbuf-2.0\2.10.0\loaders\"; Flags: ignoreversion

; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\bin\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\bin\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

