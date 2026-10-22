# Changelog

## 0.2.2

- A rule name longer than 220 characters ends in a short hash of the whole name, as AppIntBlocker
  and AppIntBlockerGUI now write it, so two programs whose names agree past the cut get their own
  rules.
- Screen readers no longer read "check mark" as part of a chosen filter chip's name.

## 0.2.1

- netSwitch has its own icon: the sidebar's switch with a traffic pulse in its track, in the accent
  on a dark tile. The README shows it too.

## 0.2.0

netSwitch is rebuilt as a network monitor with an internet switch for each app. Firewall rules
stay compatible with AppIntBlocker and AppIntBlockerGUI.

- Traffic per app, live, from Windows' own event traces: download and upload speed, totals since
  start and a chart of the last minute. The overview charts the whole PC over five minutes and
  ranks the busiest apps and hosts.
- Connections with the owning app, host names from the app's DNS lookups or reverse DNS, port and
  service, state and traffic; filters for TCP, UDP and the local network.
- An internet switch for each app that asks first, a Blocked page shared with AppIntBlocker, a
  daily firewall backup before the first change, and protection for programs in the Windows
  folder.
- A notification when an app goes online for the first time.
- A new interface on the Bugra design system, in English and Turkish, with a light and a dark
  theme.
- Removed: the restore point page, the Windows tool shortcuts and the rule editor of 0.1. The
  Windows Firewall console does those better, and the Blocked page opens it.
- CI builds the installers on Windows and runs the system tests there; a version bump on master
  publishes a release.

## 0.1.2

- Fixed duplicate network entries, the administrator checks and the traffic graph.

## 0.1.1

- Interface and network monitor fixes.

## 0.1.0

- First version: a Windows Firewall rule manager with application blocking, a network monitor,
  restore points and shortcuts to Windows network tools.
