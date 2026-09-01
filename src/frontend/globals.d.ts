declare global {
  interface SearchState {
    matches: HTMLElement[];
    currentIndex: number;
    query: string;
  }

  interface ShortcutPayload {
    key: string;
    code: string;
    ctrlKey: boolean;
    metaKey: boolean;
    altKey: boolean;
    shiftKey: boolean;
  }

  type DocSyncAction =
    | { action: "set_markdown"; content: string }
    | { action: "toggle_checkbox"; index: number; checked: boolean };

  interface Window {
    copyTabContent: (tabId: string) => void;
    copyCodeSnippet: (btn: HTMLElement) => void;
    slugifyHeading: (text: string) => string;
    scrollToSection: (id: string, line?: number) => void;
    clearSearchHighlights: () => void;
    highlightSearchMatches: (query: string) => void;
    activateMatch: (index: number) => void;
    searchNextMatch: () => void;
    searchPrevMatch: () => void;
    updateSearchCountUI: (current: number, total: number) => void;
    _searchState: SearchState;
    wrapSourceSelection: (
      prefix: string,
      suffix: string,
      defaultText: string,
    ) => void;
    insertSourceLinePrefix: (prefix: string) => void;
    insertSourceSnippet: (snippet: string) => void;
    handleSourceTab: (e: KeyboardEvent) => void;
    handleSourceEnter: (e: KeyboardEvent) => void;
    handleSourceEnterKeyup: (e: KeyboardEvent) => void;
    pushEditorHistory: (
      value: string,
      selStart: number,
      selEnd: number,
      immediate?: boolean,
    ) => void;
    editorUndo: () => void;
    editorRedo: () => void;
    syncWysiwygContent: () => void;
    debouncedSyncWysiwyg: (delayMs?: number) => void;
    formatWysiwyg: (cmd: string, val?: string | null) => void;
    formatWysiwygHeading: (tag: string) => void;
    formatWysiwygCode: () => void;
    formatWysiwygBlockquote: () => void;
    insertWysiwygCodeBlock: () => void;
    insertWysiwygTable: () => void;
    insertWysiwygCallout: (type?: string) => void;
    insertWysiwygTaskList: () => void;
    promptWysiwygLink: () => void;
    promptWysiwygImage: () => void;
    flushWysiwygContent: () => void;
    prepareDocumentModeChange: () => void;
    serializeWysiwygToMarkdown: () => string | null;
    updateToolbarActiveStates: () => void;
    onEditorSourceScroll: () => void;
    onSplitPreviewScroll: () => void;
    bindEditorScroll: (tabId?: string | number | null) => void;
    bindViewerScroll: (tabId?: string | number | null) => void;
    refreshTocTreePath: () => void;
    onViewerScroll: () => void;
    syncGutterLineHeights: (textarea: HTMLTextAreaElement | null) => void;
    syncGutterToTextarea: (textarea: HTMLTextAreaElement | null) => void;
    saveCurrentScroll: (tabId?: string | number | null) => void;
    restoreScrollState: (tabId?: string | number | null) => void;
    __ctxHref: string;
    __globalShortcutHandler: ((payload: ShortcutPayload) => void) | null;
    __docSyncHandler: ((action: DocSyncAction) => void) | undefined;
    __wysiwygChangeHandler: ((md: string) => void) | undefined;
    __recordingShortcut: boolean | undefined;
    _lastWysiwygMd: string | undefined;
    _userIsHoveringToc?: boolean;
    dioxus?: {
      send: (msg: unknown) => void;
    };
  }
}

export {};
