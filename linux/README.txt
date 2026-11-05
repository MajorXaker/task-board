TaskBoard Linux client (x64)
============================

Run without installing:
  ./taskboard                        # first run asks for the server URL
  ./taskboard --url http://host:8080  # or set the URL directly
  TASKBOARD_URL=http://host:8080 ./taskboard

The URL is remembered in ~/.config/taskboard/url.

Install (binary + application menu entry + icon):
  ./install.sh                       # to ~/.local, no sudo needed
  sudo ./install.sh                  # system-wide, to /usr/local

Requirements: webkit2gtk-4.1 and gtk3
(Arch: sudo pacman -S webkit2gtk-4.1 gtk3)

Note: taskboard.desktop is a menu entry template used by install.sh,
not an executable file.
