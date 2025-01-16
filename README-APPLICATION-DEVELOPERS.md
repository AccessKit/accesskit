# Tips for application developers

## Introduction

Please note: This document is a work in progress and will be expanded over time to include details
about how to make accessible applications using AccessKit.

## Other Resources

Many accessible technologies have their own tips for application developers.  See:

* [Orca's Tips for Application Developers](https://gitlab.gnome.org/GNOME/orca/-/blob/main/README-APPLICATION-DEVELOPERS.md?ref_type=heads&plain=0)
* [Apple's accesibility HIG](https://developer.apple.com/design/human-interface-guidelines/accessibility)

## Using screen readers

The best way to test applications for accessibility is with a screen reader.  Here are some key commands to know:

### macOS

* Turn VoiceOver on or off with `Command + F5`  (Some Macs require `Command + Function + F5`.  If your Mac or Magic Keyboard has Touch ID, press and hold the Command key while you quickly press Touch ID three times.)
* Move the VoiceOver cursor with `Control + Option + Arrow keys`, or by clicking with the mouse.
* Perform the default action with `Control + Option + Space`.
* Access a menu with `Control + Option + H + H` (pressing the H key twice).  This provides a menu of all commands including developer ones.
* More commands [here](https://support.apple.com/guide/voiceover/general-commands-cpvokys01/mac).

### Linux

orca can be enabled from the settings screen in many desktop environments, as well as by running `orca` on the command line.  `orca --setup` will
open a configuration pane, and `orca --debug-file=file.log` will log debug information to a file which can be useful for
debugging applications.

orca's keyboard configurations come in two flavors, the 'Desktop' (default) and 'Laptop' configurations. This document discusses
the Desktop flavor.

* 'Where am I?' is `Keypad Enter` (press twice for more info)
* Move focus with `Insert + Ctrl + Arrow key`
* Speak current item with `Insert + Shift + Up Arrow`
* Present object actions with `Insert-Shift-a`
* Speak entire window with Flat Review with `Keypad Add + Keypad Add` (pressing the plus key twice)
* To get a menu of all shortcuts on your system, use `Insert + H` to enter learn mode, and then `F2`.
* More information about Orca is available [here](https://help.gnome.org/users/orca/stable/index.html.en).

Problems with orca can sometimes be resolved by removing user settings at `~/.local/share/orca` and relaunching orca.


## Using accessibility inspectors

Accessibility inspectors provide a convenient way to inspect your accessibility tree.  However, they can be very
misleading in terms of how screen reader users experience an application.

### macOS

on macOS you can use the Accessibility Inspector, which is part of Xcode.  See the documentation at [Apple's developer site](https://developer.apple.com/documentation/accessibility/accessibility-inspector).

Accessibilty Inspector provides GUI buttons for navigation through the application and visually displays properties.  It includes
some auditing tools to look for common issues in your application.

The macOS tooling is of reasonably high quality for common tasks but does not always surface issues that are important
for screen reader users.

### Linux

The GNOME project provides the [accerciser](https://help.gnome.org/users/accerciser/stable/introduction.html.en) inspector.
It provides a basic tree view of the accessibility tree and can be used to inspect properties of nodes and run
some simple audits.

accerciser has many limitations.  It often fails to show node properties in complex trees and cannot display the tree
for the orca setup window for example.  It often hangs and crashes on complex trees, it does not provide a facility to focus or navigate nodes in the tree or send
most events to applications. And it will not display
trees created by accesskit on its own because it does not enable the ATSPI bus.  (To do this, turn on a screenreader or
run this command: `busctl --user set-property org.a11y.Bus /org/a11y/bus org.a11y.Status IsEnabled b true`

The KDE project provides the [Accessibility Inspector](https://apps.kde.org/accessibilityinspector/) application.  In general
it is superior to accerciser, it can display the tree for orca's setup window, it provides a checkbox to enable the ATSPI bus,
and it supports focusing and navigating nodes in the tree to some extent.  However it still frequently crashes and hangs
and provides no audit functionality.

Both inspectors often display information that is totally inaccessible to screen reader users and so relying on them
to test accessibility can be misleading.

## About virtualization

Many developers use virtualization to test their software on other platforms.  However, using virtualization to test accessibility
has some unique challenges.

### Sound

Linux often has choppy audio under virtualization due to its small audio buffers, and this may make it difficult to hear if
a screen reader is reading or to understand its speech.  Placing the following in
`~/.config/wireplumber/wireplumber.conf.d/50-alsa-config.conf` can help:

```
monitor.alsa.rules = [
  {
    matches = [
      # This matches the value of the 'node.name' property of the node.
      {
        node.name = "~alsa_output.*"
      }
    ]
    actions = {
      # Apply all the desired node specific settings here.
      update-props = {
        api.alsa.period-size   = 1024
        api.alsa.headroom      = 8192
      }
    }
  }
]
```

### Input

Popular virtualization software interferes with the keyboard shortcuts used with screen readers.  This can cause
screen readers to seemingly not respond, or respond incorrectly or inconsistently to keyboard commands.

One workaround is to mount a keyboard for the exclusive use of the guest.  (A full-size keyboard is recommended since
screen readers use many keys.)  This feature is disabled by default in VMWare; to enable it, append the following
to the vmx file:

```angular2html
usb.generic.allowHID = "TRUE"
usb.generic.allowLastHID = "TRUE"
```