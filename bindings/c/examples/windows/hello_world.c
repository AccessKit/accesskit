#include <stdio.h>
#include <windows.h>

#include "accesskit.h"

const WCHAR CLASS_NAME[] = L"AccessKitTest";

const WCHAR WINDOW_TITLE[] = L"Hello world";

const accesskit_node_id WINDOW_ID = 0;
const accesskit_node_id BUTTON_1_ID = 1;
const accesskit_node_id BUTTON_2_ID = 2;
const accesskit_node_id ANNOUNCEMENT_ID = 3;
#define INITIAL_FOCUS BUTTON_1_ID

const accesskit_rect BUTTON_1_RECT = {20.0, 20.0, 100.0, 60.0};

const accesskit_rect BUTTON_2_RECT = {20.0, 60.0, 100.0, 100.0};

const uint32_t SET_FOCUS_MSG = WM_USER;
const uint32_t DO_DEFAULT_ACTION_MSG = WM_USER + 1;

accesskit_node *build_button(accesskit_node_id id, const char *name) {
  accesskit_rect rect;
  if (id == BUTTON_1_ID) {
    rect = BUTTON_1_RECT;
  } else {
    rect = BUTTON_2_RECT;
  }

  accesskit_node_builder *builder =
      accesskit_node_builder_new(ACCESSKIT_ROLE_BUTTON);
  accesskit_node_builder_set_bounds(builder, rect);
  accesskit_node_builder_set_name(builder, name);
  accesskit_node_builder_add_action(builder, ACCESSKIT_ACTION_FOCUS);
  accesskit_node_builder_set_default_action_verb(
      builder, ACCESSKIT_DEFAULT_ACTION_VERB_CLICK);
  return accesskit_node_builder_build(builder);
}

accesskit_node *build_announcement(const char *text) {
  accesskit_node_builder *builder =
      accesskit_node_builder_new(ACCESSKIT_ROLE_STATIC_TEXT);
  accesskit_node_builder_set_name(builder, text);
  accesskit_node_builder_set_live(builder, ACCESSKIT_LIVE_POLITE);
  return accesskit_node_builder_build(builder);
}

struct window_state {
  accesskit_windows_adapter *adapter;
  accesskit_node_id focus;
  const char *announcement;
};

void window_state_free(struct window_state *state) {
  accesskit_windows_adapter_free(state->adapter);
  free(state);
}

accesskit_node *window_state_build_root(struct window_state *state) {
  accesskit_node_builder *builder =
      accesskit_node_builder_new(ACCESSKIT_ROLE_WINDOW);
  accesskit_node_builder_push_child(builder, BUTTON_1_ID);
  accesskit_node_builder_push_child(builder, BUTTON_2_ID);
  if (state->announcement != NULL) {
    accesskit_node_builder_push_child(builder, ANNOUNCEMENT_ID);
  }
  return accesskit_node_builder_build(builder);
}

accesskit_tree_update *build_initial_tree(void *userdata) {
  struct window_state *state = userdata;
  accesskit_node *root = window_state_build_root(state);
  accesskit_node *button_1 = build_button(BUTTON_1_ID, "Button 1");
  accesskit_node *button_2 = build_button(BUTTON_2_ID, "Button 2");
  accesskit_tree_update *result = accesskit_tree_update_with_capacity_and_focus(
      (state->announcement != NULL) ? 4 : 3, state->focus);
  accesskit_tree *tree = accesskit_tree_new(WINDOW_ID);
  accesskit_tree_set_app_name(tree, "Hello World");
  accesskit_tree_update_set_tree(result, tree);
  accesskit_tree_update_push_node(result, WINDOW_ID, root);
  accesskit_tree_update_push_node(result, BUTTON_1_ID, button_1);
  accesskit_tree_update_push_node(result, BUTTON_2_ID, button_2);
  if (state->announcement != NULL) {
    accesskit_node *announcement = build_announcement(state->announcement);
    accesskit_tree_update_push_node(result, ANNOUNCEMENT_ID, announcement);
  }
  return result;
}

void do_action(const accesskit_action_request *request, void *userdata) {
  HWND window = userdata;
  if (request->action == ACCESSKIT_ACTION_FOCUS) {
    LPARAM lparam = (LPARAM)(request->target);
    PostMessage((HWND)window, SET_FOCUS_MSG, 0, lparam);
  } else if (request->action == ACCESSKIT_ACTION_DEFAULT) {
    LPARAM lparam = (LPARAM)(request->target);
    PostMessage((HWND)window, DO_DEFAULT_ACTION_MSG, 0, lparam);
  }
}

accesskit_tree_update *build_tree_update_for_focus_update(void *userdata) {
  struct window_state *state = userdata;
  accesskit_tree_update *update =
      accesskit_tree_update_with_focus(state->focus);
  return update;
}

void window_state_set_focus(struct window_state *state,
                            accesskit_node_id focus) {
  state->focus = focus;
  accesskit_windows_queued_events *events =
      accesskit_windows_adapter_update_if_active(
          state->adapter, build_tree_update_for_focus_update, state);
  if (events != NULL) {
    accesskit_windows_queued_events_raise(events);
  }
}

accesskit_tree_update *build_tree_update_for_button_press(void *userdata) {
  struct window_state *state = userdata;
  accesskit_node *announcement = build_announcement(state->announcement);
  accesskit_node *root = window_state_build_root(state);
  accesskit_tree_update *update =
      accesskit_tree_update_with_capacity_and_focus(2, state->focus);
  accesskit_tree_update_push_node(update, ANNOUNCEMENT_ID, announcement);
  accesskit_tree_update_push_node(update, WINDOW_ID, root);
  return update;
}

void window_state_press_button(struct window_state *state,
                               accesskit_node_id id) {
  const char *text;
  if (id == BUTTON_1_ID) {
    text = "You pressed button 1";
  } else {
    text = "You pressed button 2";
  }
  state->announcement = text;
  accesskit_windows_queued_events *events =
      accesskit_windows_adapter_update_if_active(
          state->adapter, build_tree_update_for_button_press, state);
  if (events != NULL) {
    accesskit_windows_queued_events_raise(events);
  }
}

struct window_state *get_window_state(HWND window) {
  return (struct window_state *)(GetWindowLongPtr(window, GWLP_USERDATA));
}

void update_window_focus_state(HWND window, bool is_focused) {
  struct window_state *state = get_window_state(window);
  accesskit_windows_queued_events *events =
      accesskit_windows_adapter_update_window_focus_state(state->adapter,
                                                          is_focused);
  if (events != NULL) {
    accesskit_windows_queued_events_raise(events);
  }
}

struct window_create_params {
  accesskit_node_id initial_focus;
};

LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
  if (msg == WM_NCCREATE) {
    CREATESTRUCT *create_struct = (CREATESTRUCT *)lParam;
    struct window_create_params *create_params =
        (struct window_create_params *)create_struct->lpCreateParams;
    struct window_state *state = malloc(sizeof(struct window_state));
    state->adapter =
        accesskit_windows_adapter_new(hwnd, false, do_action, (void *)hwnd);
    state->focus = create_params->initial_focus;
    state->announcement = NULL;
    SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)state);
    return DefWindowProc(hwnd, msg, wParam, lParam);
  } else if (msg == WM_PAINT) {
    ValidateRect(hwnd, NULL);
  } else if (msg == WM_DESTROY) {
    LONG_PTR ptr = SetWindowLongPtr(hwnd, GWLP_USERDATA, 0);
    if (ptr != 0) {
      struct window_state *state = (struct window_state *)ptr;
      window_state_free(state);
    }
    PostQuitMessage(0);
  } else if (msg == WM_GETOBJECT) {
    struct window_state *state = get_window_state(hwnd);
    if (state == NULL) {
      // We need to be prepared to gracefully handle WM_GETOBJECT
      // while the window is being destroyed; this can happen if
      // the thread is using a COM STA.
      return DefWindowProc(hwnd, msg, wParam, lParam);
    }
    accesskit_opt_lresult result =
        accesskit_windows_adapter_handle_wm_getobject(
            state->adapter, wParam, lParam, build_initial_tree, state);
    if (result.has_value) {
      return result.value;
    } else {
      return DefWindowProc(hwnd, msg, wParam, lParam);
    }
  } else if (msg == WM_SETFOCUS || msg == WM_EXITMENULOOP ||
             msg == WM_EXITSIZEMOVE) {
    update_window_focus_state(hwnd, true);
  } else if (msg == WM_KILLFOCUS || msg == WM_ENTERMENULOOP ||
             msg == WM_ENTERSIZEMOVE) {
    update_window_focus_state(hwnd, false);
  } else if (msg == WM_KEYDOWN) {
    if (wParam == VK_TAB) {
      struct window_state *state = get_window_state(hwnd);
      accesskit_node_id new_focus =
          (state->focus == BUTTON_1_ID) ? BUTTON_2_ID : BUTTON_1_ID;
      window_state_set_focus(state, new_focus);
    } else if (wParam == VK_SPACE) {
      struct window_state *state = get_window_state(hwnd);
      accesskit_node_id id = state->focus;
      window_state_press_button(state, id);
    } else {
      return DefWindowProc(hwnd, msg, wParam, lParam);
    }
  } else if (msg == SET_FOCUS_MSG) {
    accesskit_node_id id = (accesskit_node_id)lParam;
    if (id == BUTTON_1_ID || id == BUTTON_2_ID) {
      struct window_state *state = get_window_state(hwnd);
      window_state_set_focus(state, id);
    }
  } else if (msg == DO_DEFAULT_ACTION_MSG) {
    accesskit_node_id id = (accesskit_node_id)lParam;
    if (id == BUTTON_1_ID || id == BUTTON_2_ID) {
      struct window_state *state = get_window_state(hwnd);
      window_state_press_button(state, id);
    }
  } else {
    return DefWindowProc(hwnd, msg, wParam, lParam);
  }
  return 0;
}

HWND create_window(const WCHAR *title, accesskit_node_id initial_focus) {
  struct window_create_params create_params;
  create_params.initial_focus = initial_focus;

  return CreateWindowEx(WS_EX_CLIENTEDGE, CLASS_NAME, title,
                        WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
                        CW_USEDEFAULT, CW_USEDEFAULT, NULL, NULL,
                        GetModuleHandle(NULL), &create_params);
}

int main() {
  printf("This example has no visible GUI, and a keyboard interface:\n");
  printf("- [Tab] switches focus between two logical buttons.\n");
  printf(
      "- [Space] 'presses' the button, adding static text in a live region "
      "announcing that it was pressed.\n");
  printf(
      "Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older "
      "versions of Windows).\n");

  WNDCLASSEX wc;
  HWND hwnd;
  MSG Msg;

  wc.cbSize = sizeof(WNDCLASSEX);
  wc.style = CS_HREDRAW | CS_VREDRAW;
  wc.lpfnWndProc = WndProc;
  wc.cbClsExtra = 0;
  wc.cbWndExtra = 0;
  wc.hInstance = GetModuleHandle(NULL);
  wc.hIcon = LoadIcon(NULL, IDI_APPLICATION);
  wc.hCursor = LoadCursor(NULL, IDC_ARROW);
  wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
  wc.lpszMenuName = NULL;
  wc.lpszClassName = CLASS_NAME;
  wc.hIconSm = LoadIcon(NULL, IDI_APPLICATION);

  if (!RegisterClassEx(&wc)) {
    return 0;
  }

  hwnd = create_window(WINDOW_TITLE, INITIAL_FOCUS);

  if (hwnd == NULL) {
    return 0;
  }

  ShowWindow(hwnd, SW_SHOW);
  UpdateWindow(hwnd);

  while (GetMessage(&Msg, NULL, 0, 0) > 0) {
    TranslateMessage(&Msg);
    DispatchMessage(&Msg);
  }
  return Msg.wParam;
}
