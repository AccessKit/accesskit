import { display } from '@kit.ArkUI';
import { inputMethod } from '@kit.IMEKit';
import showcase from 'libentry.so';

const XCOMPONENT_ID = 'accesskit_showcase_xcomponent';

@Entry
@Component
struct Index {
  private timerId: number = -1;
  private imeController: inputMethod.InputMethodController = inputMethod.getController();
  private imeAttached: boolean = false;

  aboutToDisappear(): void {
    if (this.timerId !== -1) {
      clearTimeout(this.timerId);
      this.timerId = -1;
    }
    if (this.imeAttached) {
      this.imeController.detach();
      this.imeAttached = false;
    }
  }

  private syncState(): void {
    const inputType = showcase.focusedTextInputType();
    if (inputType > 0) {
      this.attachIme(inputType);
      if (showcase.takeShowKeyboard()) {
        this.imeController.showTextInput();
      }
    } else if (this.imeAttached) {
      this.imeController.detach();
      this.imeAttached = false;
    }

    if (this.timerId !== -1) {
      clearTimeout(this.timerId);
      this.timerId = -1;
    }
    const delayMs = showcase.nextMessageDelayMs();
    if (delayMs >= 0) {
      this.timerId = setTimeout(() => {
        this.timerId = -1;
        showcase.processDueMessages();
        this.syncState();
      }, delayMs);
    }
  }

  private attachIme(inputType: number): void {
    let textInputType = inputMethod.TextInputType.TEXT;
    let enterKeyType = inputMethod.EnterKeyType.DONE;
    if (inputType === 2) {
      textInputType = inputMethod.TextInputType.EMAIL_ADDRESS;
    } else if (inputType === 3) {
      textInputType = inputMethod.TextInputType.VISIBLE_PASSWORD;
    } else if (inputType === 4) {
      textInputType = inputMethod.TextInputType.MULTILINE;
      enterKeyType = inputMethod.EnterKeyType.NEWLINE;
    }
    const textConfig: inputMethod.TextConfig = {
      inputAttribute: {
        textInputType,
        enterKeyType
      }
    };
    this.imeController.attach(false, textConfig).then(() => {
      if (!this.imeAttached) {
        this.imeAttached = true;
        this.imeController.on('insertText', (text: string) => {
          showcase.onImeText(text, false);
          this.syncState();
        });
        this.imeController.on('deleteLeft', (length: number) => {
          showcase.onImeDelete(length, 0);
          this.syncState();
        });
        this.imeController.on('deleteRight', (length: number) => {
          showcase.onImeDelete(0, length);
          this.syncState();
        });
        this.imeController.on('sendFunctionKey', (functionKey: inputMethod.FunctionKey) => {
          if (functionKey.enterKeyType === inputMethod.EnterKeyType.NEWLINE) {
            showcase.onImeText('\n', false);
          } else {
            showcase.onImeSubmit();
          }
          this.syncState();
        });
      }
    });
  }

  build() {
    Stack() {
      XComponent({
        id: XCOMPONENT_ID,
        type: XComponentType.SURFACE,
        libraryname: 'entry'
      })
        .focusable(true)
        .defaultFocus(true)
        .width('100%')
        .height('100%')
        .onFocus(() => {
          showcase.onFocusChange(true);
          this.syncState();
        })
        .onBlur(() => {
          showcase.onFocusChange(false);
          this.syncState();
        })
        .onAreaChange((_oldValue: Area, newValue: Area) => {
          const density = display.getDefaultDisplaySync().densityPixels;
          const widthPx = Number(newValue.width) * density;
          const heightPx = Number(newValue.height) * density;
          showcase.setViewport(density, 0, 0, widthPx, heightPx);
          this.syncState();
        })
        .onKeyEvent((event: KeyEvent) => {
          const action = event.type === KeyType.Down ? 0 : 1;
          const unicode = event.unicode ?? 0;
          const shift = event.getModifierKeyState ? event.getModifierKeyState(['Shift']) : false;
          const ctrl = event.getModifierKeyState ? event.getModifierKeyState(['Ctrl']) : false;
          const handled = showcase.onKeyEvent(action, event.keyCode, unicode, shift, ctrl);
          this.syncState();
          return handled;
        })
    }
    .width('100%')
    .height('100%')
  }
}
