#!/bin/bash
set -e

echo "Starting dbus..."
service dbus start

echo "Starting avahi-daemon..."
service avahi-daemon start

# --- ADD commands to start PostgreSQL, guacd, Tomcat HERE ---
# Example (adjust as needed):
# echo "Starting PostgreSQL..."
# service postgresql start
# sleep 5 # Give PG time to start
# echo "Starting Guacamole Daemon (guacd)..."
# guacd -b 0.0.0.0 -L info -f
# sleep 2
# echo "Starting Tomcat (Guacamole Web)..."
# service tomcat8 start
# sleep 5 # Give Tomcat time to start
# --- End required Guacamole service additions ---

echo "Starting Xvfb on :1..."
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

echo "Access noVNC at http://<host>:6901/"

# --- Add this line to start your Rust application ---
echo "Starting Phoxi Control Interface (Rust)..."
# Replace 'phoxi_ci_binary_name' with the actual name of your compiled executable
/usr/local/src/photoneo_campx/phoxi_control_interface_redis/target/release/phoxi_control_interface_redis &
# --- End Rust application start ---

# Keep container running
tail -f /dev/null