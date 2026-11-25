[Setup]
AppId=CANexusStudio.VirtualDesktop
AppName=CANexus Studio
AppVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\CANexus Studio
DefaultGroupName=CANexus Studio
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename=CANexus-Studio-{#AppVersion}-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Files]
Source: "{#Stage}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\CANexus Studio"; Filename: "{app}\canexus-studio.exe"
