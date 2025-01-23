#!/bin/bash
set -e

echo "Starting dbus..."
service dbus start

echo "Starting avahi-daemon..."
service avahi-daemon start

# Start Xvfb
Xvfb :1 -screen 0 1280x800x24 -ac &
sleep 2
export DISPLAY=:1

# Start x11vnc
x11vnc -display :1 -forever -nopw -listen 0.0.0.0 -rfbport 5901 &
sleep 2

# Start noVNC proxy on port 6901
websockify --web /usr/share/novnc 6901 localhost:5901 &
sleep 2

# Launch PhoXiControl (GUI) inside the X environment
echo "Starting PhoXiControl..."
DISPLAY=:1 PhoXiControl &

echo "Access noVNC at http://<host>:6901/"
tail -f /dev/null