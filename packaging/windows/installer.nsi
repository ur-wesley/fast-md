!include "MUI2.nsh"
!cd "${__FILEDIR__}/../.."

!define PRODUCT_NAME "Fast-MD"
!define PRODUCT_PUBLISHER "ur-wesley"
!define PRODUCT_WEB_SITE "https://github.com/ur-wesley/fast-md"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\fast-md.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

; Per-user installation (no admin privileges required)
RequestExecutionLevel user

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "${OUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\Fast-MD"
InstallDirRegKey HKCU "${PRODUCT_DIR_REGKEY}" ""

; UI Settings
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\fast-md.exe"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite on
  File "${BIN_PATH}"
  File "README.md"
  File "LICENSE"

  CreateDirectory "$SMPROGRAMS\Fast-MD"
  CreateShortCut "$SMPROGRAMS\Fast-MD\Fast-MD.lnk" "$INSTDIR\fast-md.exe"
  CreateShortCut "$DESKTOP\Fast-MD.lnk" "$INSTDIR\fast-md.exe"
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninst.exe"
  WriteRegStr HKCU "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\fast-md.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\fast-md.exe"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr HKCU "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
SectionEnd

Section Uninstall
  Delete "$DESKTOP\Fast-MD.lnk"
  Delete "$SMPROGRAMS\Fast-MD\Fast-MD.lnk"
  RMDir "$SMPROGRAMS\Fast-MD"

  Delete "$INSTDIR\fast-md.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninst.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKCU "${PRODUCT_DIR_REGKEY}"
  SetAutoClose true
SectionEnd
