#include <SDL.h>
#include <SDL_syswm.h>
#include <stdio.h>
#include <stdlib.h>

#include "accesskit.h"

#if (defined(__linux__) || defined(__DragonFly__) || defined(__FreeBSD__) || \
     defined(__NetBSD__) || defined(__OpenBSD__))
#define UNIX
#endif

const char WINDOW_TITLE[] = "Hello world";

static accesskit_node_id WINDOW_ID;
static accesskit_node_id BUTTON_1_ID;
static accesskit_node_id BUTTON_2_ID;
static accesskit_node_id ANNOUNCEMENT_ID;
static accesskit_node_id INITIAL_FOCUS;

const accesskit_rect BUTTON_1_RECT = {20.0, 20.0, 100.0, 60.0};

const accesskit_rect BUTTON_2_RECT = {20.0, 60.0, 100.0, 100.0};

const Sint32 SET_FOCUS_MSG = 0;
const Sint32 DO_DEFAULT_ACTION_MSG = 1;

const bool node_id_cmp(const accesskit_node_id *id1,
                       const accesskit_node_id *id2) {
  return memcmp(id1, id2, sizeof(accesskit_node_id)) == 0;
}

accesskit_node_id *node_id_dup(const accesskit_node_id *src) {
  accesskit_node_id *result = malloc(sizeof(accesskit_node_id));
  memcpy(result, src, sizeof(accesskit_node_id));
  return result;
}

accesskit_node *build_button(accesskit_node_id id, const char *name,
                             accesskit_node_class_set *classes) {
  accesskit_rect rect;
  if (node_id_cmp(&id, &BUTTON_1_ID)) {
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
  return accesskit_node_builder_build(builder, classes);
}

accesskit_node *build_announcement(const char *text,
                                   accesskit_node_class_set *classes) {
  accesskit_node_builder *builder =
      accesskit_node_builder_new(ACCESSKIT_ROLE_STATIC_TEXT);
  accesskit_node_builder_set_name(builder, text);
  accesskit_node_builder_set_live(builder, ACCESSKIT_LIVE_POLITE);
  return accesskit_node_builder_build(builder, classes);
}

struct accesskit_sdl_adapter {
#if defined(__APPLE__)
  accesskit_macos_subclassing_adapter *adapter;
#elif defined(UNIX)
  accesskit_unix_adapter *adapter;
#elif defined(_WIN32)
  accesskit_windows_subclassing_adapter *adapter;
#endif
};

void accesskit_sdl_adapter_init(struct accesskit_sdl_adapter *adapter,
                                SDL_Window *window, const char *app_name,
                                accesskit_tree_update_factory source,
                                void *source_userdata,
                                accesskit_action_handler *handler) {
#if defined(__APPLE__)
  SDL_SysWMinfo wmInfo;
  SDL_VERSION(&wmInfo.version);
  SDL_GetWindowWMInfo(window, &wmInfo);
  adapter->adapter = accesskit_macos_subclassing_adapter_for_window(
      (void *)wmInfo.info.cocoa.window, source, source_userdata, handler);
#elif defined(UNIX)
  adapter->adapter = accesskit_unix_adapter_new(app_name, "SDL", "2.0", source,
                                                source_userdata, handler);
#elif defined(_WIN32)
  SDL_SysWMinfo wmInfo;
  SDL_VERSION(&wmInfo.version);
  SDL_GetWindowWMInfo(window, &wmInfo);
  adapter->adapter = accesskit_windows_subclassing_adapter_new(
      wmInfo.info.win.window, source, source_userdata, handler);
#endif
}

void accesskit_sdl_adapter_destroy(struct accesskit_sdl_adapter *adapter) {
  if (adapter->adapter != NULL) {
#if defined(__APPLE__)
    accesskit_macos_subclassing_adapter_free(adapter->adapter);
#elif defined(UNIX)
    accesskit_unix_adapter_free(adapter->adapter);
#elif defined(_WIN32)
    accesskit_windows_subclassing_adapter_free(adapter->adapter);
#endif
  }
}

void accesskit_sdl_adapter_update(const struct accesskit_sdl_adapter *adapter,
                                  accesskit_tree_update *update) {
#if defined(__APPLE__)
  accesskit_macos_queued_events *events =
      accesskit_macos_subclassing_adapter_update(adapter->adapter, update);
  if (events != NULL) {
    accesskit_macos_queued_events_raise(events);
  }
#elif defined(UNIX)
  accesskit_unix_adapter_update(adapter->adapter, update);
#elif defined(_WIN32)
  accesskit_windows_queued_events *events =
      accesskit_windows_subclassing_adapter_update(adapter->adapter, update);
  if (events != NULL) {
    accesskit_windows_queued_events_raise(events);
  }
#endif
}

void accesskit_sdl_adapter_update_if_active(
    const struct accesskit_sdl_adapter *adapter,
    accesskit_tree_update_factory update_factory,
    void *update_factory_userdata) {
#if defined(__APPLE__)
  accesskit_macos_queued_events *events =
      accesskit_macos_subclassing_adapter_update_if_active(
          adapter->adapter, update_factory, update_factory_userdata);
  if (events != NULL) {
    accesskit_macos_queued_events_raise(events);
  }
#elif defined(UNIX)
  accesskit_unix_adapter_update(adapter->adapter,
                                update_factory(update_factory_userdata));
#elif defined(_WIN32)
  accesskit_windows_queued_events *events =
      accesskit_windows_subclassing_adapter_update_if_active(
          adapter->adapter, update_factory, update_factory_userdata);
  if (events != NULL) {
    accesskit_windows_queued_events_raise(events);
  }
#endif
}

void accesskit_sdl_adapter_update_root_window_bounds(
    const struct accesskit_sdl_adapter *adapter, SDL_Window *window) {
#if defined(UNIX)
  int x, y, width, height;
  SDL_GetWindowPosition(window, &x, &y);
  SDL_GetWindowSize(window, &width, &height);
  int top, left, bottom, right;
  SDL_GetWindowBordersSize(window, &top, &left, &bottom, &right);
  accesskit_rect outer_bounds = {x - left, y - top, x + width + right,
                                 y + height + bottom};
  accesskit_rect inner_bounds = {x, y, x + width, y + height};
  accesskit_unix_adapter_set_root_window_bounds(adapter->adapter, outer_bounds,
                                                inner_bounds);
#endif
}

struct window_state {
  accesskit_node_id focus;
  bool is_window_focused;
  const char *announcement;
  accesskit_node_class_set *node_classes;
  SDL_mutex *mutex;
};

void window_state_init(struct window_state *state) {
  state->focus = INITIAL_FOCUS;
  state->is_window_focused = false;
  state->announcement = NULL;
  state->node_classes = accesskit_node_class_set_new();
  state->mutex = SDL_CreateMutex();
}

void window_state_destroy(struct window_state *state) {
  accesskit_node_class_set_free(state->node_classes);
  SDL_DestroyMutex(state->mutex);
}

void window_state_lock(struct window_state *state) {
  SDL_LockMutex(state->mutex);
}

void window_state_unlock(struct window_state *state) {
  SDL_UnlockMutex(state->mutex);
}

accesskit_opt_node_id window_state_focus(struct window_state *state) {
  accesskit_opt_node_id result;
  result.has_value = state->is_window_focused;
  if (result.has_value) {
    result.value = state->focus;
  }
  return result;
}

accesskit_node *window_state_build_root(struct window_state *state) {
  accesskit_node_builder *builder =
      accesskit_node_builder_new(ACCESSKIT_ROLE_WINDOW);
  accesskit_node_builder_push_child(builder, BUTTON_1_ID);
  accesskit_node_builder_push_child(builder, BUTTON_2_ID);
  if (state->announcement != NULL) {
    accesskit_node_builder_push_child(builder, ANNOUNCEMENT_ID);
  }
  accesskit_node_builder_set_name(builder, WINDOW_TITLE);
  return accesskit_node_builder_build(builder, state->node_classes);
}

accesskit_tree_update *window_state_build_initial_tree(
    struct window_state *state) {
  accesskit_node *root = window_state_build_root(state);
  accesskit_node *button_1 =
      build_button(BUTTON_1_ID, "Button 1", state->node_classes);
  accesskit_node *button_2 =
      build_button(BUTTON_2_ID, "Button 2", state->node_classes);
  accesskit_tree_update *result =
      accesskit_tree_update_new((state->announcement != NULL) ? 4 : 3);
  result->tree.has_value = true;
  result->tree.value = accesskit_tree_new(WINDOW_ID);
  result->focus = window_state_focus(state);
  result->ids[0] = WINDOW_ID;
  result->nodes[0] = root;
  result->ids[1] = BUTTON_1_ID;
  result->nodes[1] = button_1;
  result->ids[2] = BUTTON_2_ID;
  result->nodes[2] = button_2;
  if (state->announcement != NULL) {
    result->ids[3] = ANNOUNCEMENT_ID;
    result->nodes[3] =
        build_announcement(state->announcement, state->node_classes);
  }
  return result;
}

accesskit_tree_update *build_tree_update_for_button_press(void *userdata) {
  struct window_state *state = userdata;
  accesskit_node *announcement =
      build_announcement(state->announcement, state->node_classes);
  accesskit_node *root = window_state_build_root(state);
  accesskit_tree_update *update = accesskit_tree_update_new(2);
  update->ids[0] = ANNOUNCEMENT_ID;
  update->nodes[0] = announcement;
  update->ids[1] = WINDOW_ID;
  update->nodes[1] = root;
  update->focus = window_state_focus(state);
  return update;
}

void window_state_press_button(struct window_state *state,
                               const struct accesskit_sdl_adapter *adapter,
                               accesskit_node_id id) {
  const char *text;
  if (node_id_cmp(&id, &BUTTON_1_ID)) {
    text = "You pressed button 1";
  } else {
    text = "You pressed button 2";
  }
  state->announcement = text;
  accesskit_sdl_adapter_update_if_active(
      adapter, build_tree_update_for_button_press, state);
}

accesskit_tree_update *build_tree_update_for_focus_update(void *userdata) {
  struct window_state *state = userdata;
  accesskit_opt_node_id focus = window_state_focus(state);
  accesskit_tree_update *update = accesskit_tree_update_new(0);
  update->focus = focus;
  return update;
}

void window_state_update_focus(struct window_state *state,
                               const struct accesskit_sdl_adapter *adapter) {
  accesskit_sdl_adapter_update_if_active(
      adapter, build_tree_update_for_focus_update, state);
}

struct action_handler_state {
  Uint32 event_type;
  Uint32 window_id;
};

void do_action(const accesskit_action_request *request, void *userdata) {
  struct action_handler_state *state = userdata;
  SDL_Event event;
  SDL_zero(event);
  event.type = state->event_type;
  event.user.windowID = state->window_id;
  event.user.data1 = node_id_dup(&request->target);
  if (request->action == ACCESSKIT_ACTION_FOCUS) {
    event.user.code = SET_FOCUS_MSG;
    SDL_PushEvent(&event);
  } else if (request->action == ACCESSKIT_ACTION_DEFAULT) {
    event.user.code = DO_DEFAULT_ACTION_MSG;
    SDL_PushEvent(&event);
  }
}

accesskit_tree_update *build_initial_tree(void *userdata) {
  struct window_state *state = userdata;
  window_state_lock(state);
  accesskit_tree_update *update = window_state_build_initial_tree(state);
  window_state_unlock(state);
  return update;
}

int main(int argc, char *argv[]) {
  printf("This example has no visible GUI, and a keyboard interface:\n");
  printf("- [Tab] switches focus between two logical buttons.\n");
  printf(
      "- [Space] 'presses' the button, adding static text in a live region "
      "announcing that it was pressed.\n");
#if defined(_WIN32)
  printf(
      "Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older "
      "versions of Windows).\n");
#elif defined(UNIX)
  printf("Enable Orca with [Super]+[Alt]+[S].\n");
#endif
  if (SDL_Init(SDL_INIT_VIDEO) != 0) {
    fprintf(stderr, "SDL initialization failed: (%s)\n", SDL_GetError());
    return -1;
  }
  Uint32 user_event = SDL_RegisterEvents(1);
  if (user_event == (Uint32)-1) {
    fprintf(stderr, "Couldn't register user event: (%s)\n", SDL_GetError());
    return -1;
  }
  WINDOW_ID = accesskit_node_id_new(1).value;
  BUTTON_1_ID = accesskit_node_id_new(2).value;
  BUTTON_2_ID = accesskit_node_id_new(3).value;
  ANNOUNCEMENT_ID = accesskit_node_id_new(4).value;
  INITIAL_FOCUS = BUTTON_1_ID;

  struct window_state state;
  window_state_init(&state);
  SDL_Window *window =
      SDL_CreateWindow(WINDOW_TITLE, SDL_WINDOWPOS_UNDEFINED,
                       SDL_WINDOWPOS_UNDEFINED, 800, 600, SDL_WINDOW_HIDDEN);
  Uint32 window_id = SDL_GetWindowID(window);
  struct action_handler_state action_handler = {user_event, window_id};
  struct accesskit_sdl_adapter adapter;
  accesskit_sdl_adapter_init(
      &adapter, window, "hello_world", build_initial_tree, &state,
      accesskit_action_handler_new(do_action, &action_handler));
  SDL_ShowWindow(window);

  SDL_Event event;
  while (SDL_WaitEvent(&event)) {
    if (event.type == SDL_QUIT) {
      break;
    } else if (event.type == SDL_WINDOWEVENT &&
               event.window.windowID == window_id) {
      switch (event.window.event) {
        case SDL_WINDOWEVENT_FOCUS_GAINED:
          window_state_lock(&state);
          state.is_window_focused = true;
          window_state_update_focus(&state, &adapter);
          window_state_unlock(&state);
          continue;
          break;
        case SDL_WINDOWEVENT_FOCUS_LOST:
          window_state_lock(&state);
          state.is_window_focused = false;
          window_state_update_focus(&state, &adapter);
          window_state_unlock(&state);
          break;
        case SDL_WINDOWEVENT_MOVED:
          accesskit_sdl_adapter_update_root_window_bounds(&adapter, window);
          break;
        case SDL_WINDOWEVENT_SHOWN:
          accesskit_sdl_adapter_update_root_window_bounds(&adapter, window);
          break;
      }
    } else if (event.type == SDL_KEYDOWN && event.key.windowID == window_id) {
      switch (event.key.keysym.sym) {
        case SDLK_TAB:
          window_state_lock(&state);
          if (node_id_cmp(&state.focus, &BUTTON_1_ID)) {
            state.focus = BUTTON_2_ID;
          } else {
            state.focus = BUTTON_1_ID;
          }
          window_state_update_focus(&state, &adapter);
          window_state_unlock(&state);
          break;
        case SDLK_SPACE:
          window_state_lock(&state);
          accesskit_node_id id = state.focus;
          window_state_press_button(&state, &adapter, id);
          window_state_unlock(&state);
          break;
      }
    } else if (event.type == user_event && event.user.windowID == window_id &&
               (node_id_cmp(event.user.data1, &BUTTON_1_ID) ||
                node_id_cmp(event.user.data1, &BUTTON_2_ID))) {
      window_state_lock(&state);
      accesskit_node_id *target = event.user.data1;
      if (event.user.code == SET_FOCUS_MSG) {
        state.focus = *target;
        window_state_update_focus(&state, &adapter);
      } else if (event.user.code == DO_DEFAULT_ACTION_MSG) {
        window_state_press_button(&state, &adapter, *target);
      }
      window_state_unlock(&state);
    }
  }

  accesskit_sdl_adapter_destroy(&adapter);
  window_state_destroy(&state);
  SDL_Quit();
  return 0;
}
