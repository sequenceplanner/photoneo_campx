#!/bin/bash

# List details of the directories
echo "Listing permissions of required directories:"
ls -ld /tmp/.X11-unix /var/run/dbus /dev/shm

# Change permissions to allow full access
echo "Changing permissions to 777 for /tmp/.X11-unix, /var/run/dbus, and /dev/shm..."
sudo chmod 777 /tmp/.X11-unix /var/run/dbus /dev/shm

# Allow local Docker containers to access the X server
echo "Granting X server access to local Docker containers..."
sudo xhost +local:

echo "Setup completed successfully."
