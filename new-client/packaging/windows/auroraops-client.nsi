!include "MUI2.nsh"

!ifndef VERSION
!define VERSION "0.0.0"
!endif
!ifndef SOURCE_EXE
!define SOURCE_EXE "..\target\release\auroraops-agent.exe"
!endif
!ifndef ARCH
!define ARCH "x64"
!endif

Name "AuroraOps Client"
OutFile "AuroraOps-Client-Setup-${VERSION}-${ARCH}.exe"
InstallDir "$PROGRAMFILES64\AuroraOps\AuroraOps Client"
InstallDirRegKey HKLM "Software\AuroraOps\Client" "InstallDir"
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

Section "AuroraOps Client" SEC_MAIN
  SectionIn RO
  SetOutPath "$INSTDIR"
  File /oname=auroraops-agent.exe "${SOURCE_EXE}"
  WriteRegStr HKLM "Software\AuroraOps\Client" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateDirectory "$SMPROGRAMS\AuroraOps"
  CreateShortcut "$SMPROGRAMS\AuroraOps\AuroraOps Client.lnk" "$INSTDIR\auroraops-agent.exe"
  CreateShortcut "$DESKTOP\AuroraOps Client.lnk" "$INSTDIR\auroraops-agent.exe"
SectionEnd

Section "Install and start system service" SEC_SERVICE
  ExecWait '"$INSTDIR\auroraops-agent.exe" --install-service'
SectionEnd

Section "Uninstall"
  ExecWait '"$INSTDIR\auroraops-agent.exe" --uninstall-service'
  Delete "$DESKTOP\AuroraOps Client.lnk"
  Delete "$SMPROGRAMS\AuroraOps\AuroraOps Client.lnk"
  RMDir "$SMPROGRAMS\AuroraOps"
  Delete "$INSTDIR\auroraops-agent.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\AuroraOps\Client"
SectionEnd

LangString DESC_MAIN ${LANG_SIMPCHINESE} "安装 AuroraOps 客户端主程序和快捷方式。"
LangString DESC_SERVICE ${LANG_SIMPCHINESE} "注册并启动 auroraops-agent Windows 系统服务。"
LangString DESC_MAIN ${LANG_ENGLISH} "Install AuroraOps client executable and shortcuts."
LangString DESC_SERVICE ${LANG_ENGLISH} "Register and start the auroraops-agent Windows service."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_MAIN} $(DESC_MAIN)
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_SERVICE} $(DESC_SERVICE)
!insertmacro MUI_FUNCTION_DESCRIPTION_END
