# AccessKit macOS adapter

This is the macOS adapter for [AccessKit](https://accesskit.dev/). It exposes an AccessKit accessibility tree through the Cocoa `NSAccessibility` protocol.

Focus queries report the focused element within each view, including inactive
windows, as native AppKit controls do. If that element has an active descendant,
the descendant receives accessibility focus. Focus-change notifications remain
restricted to the focused host, so updates in background windows do not move
the screen reader's focus.

## Known issues

- The selected state of ListBox items is not reported ([#520](https://github.com/AccessKit/accesskit/issues/520))
