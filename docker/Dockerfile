# Use Ubuntu 16.04 as the base image
FROM ubuntu:16.04

# Switch to the root user
USER root

ENV PHOXI_CONTROL_PATH="/opt/PhoXiControl-1.2.35"

# Update and install required packages
RUN set -eux \
    && apt-get update -y \
    && apt-get install -y -q \
       software-properties-common \
       avahi-utils \
       libqt5core5a \
       libqt5dbus5 \
       libqt5gui5 \
       libgtk2.0-0 \
       libssl1.0.0 \
       libgomp1 \
       libpcre16-3 \
       libflann-dev \
       libssh2-1-dev \
       libpng16-16 \
       libglfw3-dev \
       xcb \
    && apt-add-repository universe \
    && apt-get update -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy installers into the container
COPY context/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run /tmp/
COPY context/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run /tmp/

# Ensure the installer files have execution permissions
RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run \
    && chmod +x /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# Install PhoXiControl
RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run --accept ${PHOXI_CONTROL_PATH} \
    && rm /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

RUN set -eux \
    && mkdir /root/.PhotoneoPhoXiControl

CMD ["PhoXiControl"]






# FROM ubuntu:16.04

# USER root

# RUN set -eux \
#     && apt-get update -y \
#     && apt-get install -y -q software-properties-common && apt-add-repository universe \
#     && apt-get update -y \
#     && apt install -y avahi-utils libqt5core5a libqt5dbus5 libqt5gui5 libgtk2.0-0 libssl1.0.0 libgomp1 libpcre16-3 libflann-dev libssh2-1-dev libpng16-16 libglfw3-dev xcb

# # COPY installer/phoxi.run /tmp/phoxi.run
# # COPY system_files/PhoXiControl /usr/local/bin/PhoXiControl

# # ENV PHOXI_CONTROL_PATH="/opt/Photoneo/PhoXiControl"

# # RUN set -eux \
# #     && cd /tmp \
# #     && chmod a+x phoxi.run \
# #     && ./phoxi.run --accept ${PHOXI_CONTROL_PATH} \
# #     && rm -rf phoxi.run

# # # TODO: this is a bug SCAN-3548
# # RUN set -eux \
# #     && mkdir /root/.PhotoneoPhoXiControl

# # CMD ["/usr/local/bin/PhoXiControl"]

# # chmod +x PhotoneoPhoXiControlInstaller-<version>-Ubuntu[22 20 18]-STABLE.run
# # $ sudo ./PhotoneoPhoXiControlInstaller-<version>-Ubuntu[22 20 18]-STABLE.run
# # $ sudo ./PhotoneoPhoXiControlInstaller-Ubuntu-<version>.run --accept
# # /opt/Photoneo/PhoXiControl-<version>

# # Copy your installer into the image
# COPY PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run
# COPY PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# # Ensure the installer has execution permissions
# RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run
# RUN chmod +x /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# # Run the PhoXiControl Installer
# RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run --accept ${PHOXI_CONTROL_PATH}
# RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

# # Open a bash shell by default
# CMD ["/bin/bash"]

# FROM ubuntu:16.04

# # Avoid interactive prompts during builds
# ENV DEBIAN_FRONTEND=noninteractive

# # Install common dependencies
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends \
#       curl \
#       ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # --- Phoxi Contol Installation ---
# # Copy the first .run file into the image
# COPY PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

# # Make it executable and run the installer
# RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run && \
#     yes | /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run && \
#     rm /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

# # --- Localization SDK Installation ---
# # Copy the second .run file into the image
# COPY PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# # Make it executable and run the installer
# RUN chmod +x /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run && \
#     yes | /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run --silent && \
#     rm /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# # Default command when running the container
# CMD ["/bin/bash"]
