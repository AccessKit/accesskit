export const setViewport: (
  scaleFactor: number,
  safeAreaInsetX: number,
  safeAreaInsetY: number,
  safeAreaWidth: number,
  safeAreaHeight: number
) => void;
export const onFocusChange: (isFocused: boolean) => void;
export const onKeyEvent: (
  action: number,
  keyCode: number,
  unicodeChar: number,
  shiftPressed: boolean,
  ctrlPressed: boolean
) => boolean;
export const onImeText: (text: string, isComposing: boolean) => boolean;
export const onImeDelete: (beforeCount: number, afterCount: number) => boolean;
export const onImeSubmit: () => boolean;
export const processDueMessages: () => boolean;
export const nextMessageDelayMs: () => number;
export const focusedTextInputType: () => number;
export const takeShowKeyboard: () => boolean;
