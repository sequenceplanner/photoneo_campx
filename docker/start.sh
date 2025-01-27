#!/bin/bash
set -e

echo "Starting dbus..."
service dbus start

echo "Starting avahi-daemon..."
service avahi-daemon start

echo "Starting Xvfb on :1..."
# Xvfb :1 -screen 0 1280x800x24 -ac &
Xvfb :1 -screen 0 1680x1050x24 -ac &
sleep 2
export DISPLAY=:1

echo "Starting x11vnc..."
x11vnc -display :1 -forever -nopw \
       -listen 0.0.0.0 -rfbport 5901 \
       -noxdamage -noxfixes -nowf \
       & 
sleep 2

echo "Starting xfce4..."
startxfce4 &

# Start noVNC proxy on port 6901
websockify --web /usr/share/novnc 6901 localhost:5901 &
sleep 2

echo "Starting Flask REST API server..."
# Activate Python virtual environment and run Flask in the background
source /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables/api_server/venv/bin/activate
python3 /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables/api_server/app.py &

sleep 2

echo "Access noVNC at http://<host>:6901/"
echo "Flask REST API server is running on http://<host>:5000/"

# Keep the container running
tail -f /dev/null