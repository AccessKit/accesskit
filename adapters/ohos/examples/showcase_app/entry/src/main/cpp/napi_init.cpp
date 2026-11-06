// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#include <ace/xcomponent/native_interface_xcomponent.h>
#include <arkui/native_interface_accessibility.h>
#include <napi/native_api.h>
#include <cstdint>
#include <string>

extern "C" {
struct ShowcaseState;

ShowcaseState* accesskit_ohos_showcase_create(void* provider);
void accesskit_ohos_showcase_destroy(ShowcaseState* handle);
void accesskit_ohos_showcase_set_viewport(
    ShowcaseState* handle,
    double scale_factor,
    double safe_area_inset_x,
    double safe_area_inset_y,
    double safe_area_width,
    double safe_area_height);
void accesskit_ohos_showcase_on_focus_change(ShowcaseState* handle, bool is_focused);
bool accesskit_ohos_showcase_on_key_event(
    ShowcaseState* handle,
    int32_t action,
    int32_t key_code,
    uint32_t unicode_char,
    bool shift_pressed,
    bool ctrl_pressed);
bool accesskit_ohos_showcase_on_ime_text(
    ShowcaseState* handle,
    const char* text,
    bool is_composing);
bool accesskit_ohos_showcase_on_ime_delete(
    ShowcaseState* handle,
    size_t before_count,
    size_t after_count);
bool accesskit_ohos_showcase_on_ime_submit(ShowcaseState* handle);
bool accesskit_ohos_showcase_process_due_messages(ShowcaseState* handle);
int64_t accesskit_ohos_showcase_next_message_delay_ms(ShowcaseState* handle);
int32_t accesskit_ohos_showcase_focused_text_input_type(ShowcaseState* handle);
bool accesskit_ohos_showcase_take_show_keyboard(ShowcaseState* handle);
}

namespace {
ShowcaseState* g_showcase = nullptr;
OH_NativeXComponent_Callback g_xcomponent_callbacks{};

void OnSurfaceCreated(OH_NativeXComponent* component, void* /*window*/) {
    if (g_showcase != nullptr) {
        return;
    }
    ArkUI_AccessibilityProvider* provider = nullptr;
    if (OH_NativeXComponent_GetNativeAccessibilityProvider(component, &provider) ==
            OH_NATIVEXCOMPONENT_RESULT_SUCCESS &&
        provider != nullptr) {
        g_showcase = accesskit_ohos_showcase_create(provider);
    }
}

void OnSurfaceChanged(OH_NativeXComponent* /*component*/, void* /*window*/) {}

void OnSurfaceDestroyed(OH_NativeXComponent* /*component*/, void* /*window*/) {
    if (g_showcase != nullptr) {
        accesskit_ohos_showcase_destroy(g_showcase);
        g_showcase = nullptr;
    }
}

void DispatchTouchEvent(OH_NativeXComponent* /*component*/, void* /*window*/) {}

napi_value SetViewport(napi_env env, napi_callback_info info) {
    size_t argc = 5;
    napi_value args[5];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 5 || g_showcase == nullptr) {
        return nullptr;
    }
    double scale = 1.0, x = 0.0, y = 0.0, width = 0.0, height = 0.0;
    napi_get_value_double(env, args[0], &scale);
    napi_get_value_double(env, args[1], &x);
    napi_get_value_double(env, args[2], &y);
    napi_get_value_double(env, args[3], &width);
    napi_get_value_double(env, args[4], &height);
    accesskit_ohos_showcase_set_viewport(g_showcase, scale, x, y, width, height);
    return nullptr;
}

napi_value OnFocusChange(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1 || g_showcase == nullptr) {
        return nullptr;
    }
    bool focused = false;
    napi_get_value_bool(env, args[0], &focused);
    accesskit_ohos_showcase_on_focus_change(g_showcase, focused);
    return nullptr;
}

napi_value OnKeyEvent(napi_env env, napi_callback_info info) {
    size_t argc = 5;
    napi_value args[5];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    bool handled = false;
    if (argc >= 5 && g_showcase != nullptr) {
        int32_t action = 0, key_code = 0;
        uint32_t unicode_char = 0;
        bool shift = false, ctrl = false;
        napi_get_value_int32(env, args[0], &action);
        napi_get_value_int32(env, args[1], &key_code);
        napi_get_value_uint32(env, args[2], &unicode_char);
        napi_get_value_bool(env, args[3], &shift);
        napi_get_value_bool(env, args[4], &ctrl);
        handled = accesskit_ohos_showcase_on_key_event(
            g_showcase, action, key_code, unicode_char, shift, ctrl);
    }
    napi_value result = nullptr;
    napi_get_boolean(env, handled, &result);
    return result;
}

napi_value OnImeText(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    bool handled = false;
    if (argc >= 2 && g_showcase != nullptr) {
        size_t len = 0;
        napi_get_value_string_utf8(env, args[0], nullptr, 0, &len);
        std::string text(len, '\0');
        napi_get_value_string_utf8(env, args[0], text.data(), len + 1, &len);
        bool is_composing = false;
        napi_get_value_bool(env, args[1], &is_composing);
        handled = accesskit_ohos_showcase_on_ime_text(g_showcase, text.c_str(), is_composing);
    }
    napi_value result = nullptr;
    napi_get_boolean(env, handled, &result);
    return result;
}

napi_value OnImeDelete(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    bool handled = false;
    if (argc >= 2 && g_showcase != nullptr) {
        uint32_t before = 0, after = 0;
        napi_get_value_uint32(env, args[0], &before);
        napi_get_value_uint32(env, args[1], &after);
        handled = accesskit_ohos_showcase_on_ime_delete(g_showcase, before, after);
    }
    napi_value result = nullptr;
    napi_get_boolean(env, handled, &result);
    return result;
}

napi_value OnImeSubmit(napi_env env, napi_callback_info info) {
    bool handled = g_showcase != nullptr && accesskit_ohos_showcase_on_ime_submit(g_showcase);
    napi_value result = nullptr;
    napi_get_boolean(env, handled, &result);
    return result;
}

napi_value ProcessDueMessages(napi_env env, napi_callback_info /*info*/) {
    bool updated =
        g_showcase != nullptr && accesskit_ohos_showcase_process_due_messages(g_showcase);
    napi_value result = nullptr;
    napi_get_boolean(env, updated, &result);
    return result;
}

napi_value NextMessageDelayMs(napi_env env, napi_callback_info /*info*/) {
    int64_t delay =
        g_showcase != nullptr ? accesskit_ohos_showcase_next_message_delay_ms(g_showcase) : -1;
    napi_value result = nullptr;
    napi_create_int64(env, delay, &result);
    return result;
}

napi_value FocusedTextInputType(napi_env env, napi_callback_info /*info*/) {
    int32_t type =
        g_showcase != nullptr ? accesskit_ohos_showcase_focused_text_input_type(g_showcase) : 0;
    napi_value result = nullptr;
    napi_create_int32(env, type, &result);
    return result;
}

napi_value TakeShowKeyboard(napi_env env, napi_callback_info /*info*/) {
    bool show =
        g_showcase != nullptr && accesskit_ohos_showcase_take_show_keyboard(g_showcase);
    napi_value result = nullptr;
    napi_get_boolean(env, show, &result);
    return result;
}

napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor desc[] = {
        {"setViewport", nullptr, SetViewport, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"onFocusChange", nullptr, OnFocusChange, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"onKeyEvent", nullptr, OnKeyEvent, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"onImeText", nullptr, OnImeText, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"onImeDelete", nullptr, OnImeDelete, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"onImeSubmit", nullptr, OnImeSubmit, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"processDueMessages", nullptr, ProcessDueMessages, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"nextMessageDelayMs", nullptr, NextMessageDelayMs, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"focusedTextInputType", nullptr, FocusedTextInputType, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"takeShowKeyboard", nullptr, TakeShowKeyboard, nullptr, nullptr, nullptr, napi_default, nullptr},
    };
    napi_define_properties(env, exports, sizeof(desc) / sizeof(desc[0]), desc);

    napi_value export_instance = nullptr;
    if (napi_get_named_property(env, exports, OH_NATIVE_XCOMPONENT_OBJ, &export_instance) ==
        napi_ok) {
        OH_NativeXComponent* xcomponent = nullptr;
        if (napi_unwrap(env, export_instance, reinterpret_cast<void**>(&xcomponent)) == napi_ok &&
            xcomponent != nullptr) {
            g_xcomponent_callbacks.OnSurfaceCreated = OnSurfaceCreated;
            g_xcomponent_callbacks.OnSurfaceChanged = OnSurfaceChanged;
            g_xcomponent_callbacks.OnSurfaceDestroyed = OnSurfaceDestroyed;
            g_xcomponent_callbacks.DispatchTouchEvent = DispatchTouchEvent;
            OH_NativeXComponent_RegisterCallback(xcomponent, &g_xcomponent_callbacks);
        }
    }
    return exports;
}
}  // namespace

static napi_module g_module = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "entry",
    .nm_priv = nullptr,
    .reserved = {0},
};

extern "C" __attribute__((constructor)) void RegisterEntryModule(void) {
    napi_module_register(&g_module);
}
