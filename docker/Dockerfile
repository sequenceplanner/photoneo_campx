FROM ubuntu:16.04

# Noninteractive install
ENV DEBIAN_FRONTEND=noninteractive

# Update OS and install prerequisites
RUN apt-get update \
    && apt-get dist-upgrade -y \
    && apt-get install -y software-properties-common

# Install packages
RUN add-apt-repository ppa:ts.sch.gr/ppa -y \
    && apt-get update \
    && echo debconf shared/accepted-oracle-license-v1-1 select true | debconf-set-selections \
    && echo debconf shared/accepted-oracle-license-v1-1 seen true | debconf-set-selections \
    && apt-get install -y \
       oracle-java8-installer \
       oracle-java8-set-default \
       tomcat8 tomcat8-admin tomcat8-common tomcat8-docs tomcat8-user \
       libcairo2-dev libjpeg-turbo8-dev libpng12-dev libossp-uuid-dev \
       libfreerdp-dev libpango1.0-dev libssh2-1-dev libtelnet-dev libvncserver-dev \
       libpulse-dev libssl-dev libvorbis-dev libwebp-dev build-essential autoconf libtool \
       maven pwgen git \
       postgresql-9.5 postgresql-server-dev-9.5 postgresql-common \
       ghostscript wget haveged \
       nginx

# Create Guacamole directories & files
RUN mkdir -p /etc/guacamole \
    && mkdir -p /etc/guacamole/lib \
    && mkdir -p /etc/guacamole/extensions \
    && touch /etc/guacamole/user-mapping.xml

# Configure GUACAMOLE_HOME for Tomcat
RUN echo "" >> /etc/default/tomcat8 \
    && echo "# GUACAMOLE ENV VARIABLE" >> /etc/default/tomcat8 \
    && echo "GUACAMOLE_HOME=/etc/guacamole" >> /etc/default/tomcat8

# Build and install Guacamole server
WORKDIR /opt
RUN git clone https://github.com/apache/incubator-guacamole-server.git \
    && cd incubator-guacamole-server \
    && autoreconf -fi \
    && ./configure --with-init-dir=/etc/init.d \
    && make \
    && make install \
    && ldconfig

# Remove the old Maven if present, and install prerequisites
RUN apt-get remove -y maven && \
    apt-get update && \
    apt-get install -y curl

# Need a very specific Maven version, install Maven 3.6.3 manually
ENV MAVEN_VERSION=3.6.3
RUN wget --no-verbose -O /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz \
      https://archive.apache.org/dist/maven/maven-3/3.6.3/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz \
    && tar -xzf /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz -C /opt \
    && ln -s /opt/apache-maven-${MAVEN_VERSION} /opt/maven \
    && ln -s /opt/maven/bin/mvn /usr/bin/mvn

ENV MAVEN_HOME=/opt/maven
ENV PATH=$MAVEN_HOME/bin:$PATH

# Build and install Guacamole client (web app)
WORKDIR /opt
RUN git clone --branch 0.9.12-incubating https://github.com/apache/incubator-guacamole-client.git \
    && cd incubator-guacamole-client \
    && mvn package \
    && cp ./guacamole/target/guacamole-0.9.12-incubating.war /var/lib/tomcat8/webapps/guacamole.war \
    && cp ./extensions/guacamole-auth-jdbc/modules/guacamole-auth-jdbc-postgresql/target/guacamole-auth-jdbc-postgresql-0.9.12-incubating.jar \
          /etc/guacamole/extensions/

# Install PostgreSQL JDBC driver
WORKDIR /usr/local/src
RUN wget -c https://jdbc.postgresql.org/download/postgresql-42.0.0.jar \
    && cp postgresql-42.0.0.jar /etc/guacamole/lib/

# Link FreeRDP libraries (as in your script)
RUN ln -s /usr/local/lib/freerdp/* /usr/lib/x86_64-linux-gnu/freerdp/. || true

# Generate a random password example
RUN pwgen -n 14

# Configure Guacamole properties
RUN echo "#api-session-timeout: 60" >> /etc/guacamole/guacamole.properties \
    && echo "available-languages: en" >> /etc/guacamole/guacamole.properties \
    && echo "guacd-hostname: localhost" >> /etc/guacamole/guacamole.properties \
    && echo "guacd-port: 4822" >> /etc/guacamole/guacamole.properties \
    && echo "#guacd-ssl: true" >> /etc/guacamole/guacamole.properties \
    && echo "lib-directory: /var/lib/tomcat8/webapps/guacamole/WEB-INF/classes" >> /etc/guacamole/guacamole.properties \
    && echo "" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-hostname: localhost" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-port: 5432" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-database: guacamole_db" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-username: guacamole_user" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-password: PASSWORD" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-user-password-min-length: 8" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-user-password-require-multiple-case: true" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-user-password-require-symbol: true" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-user-password-require-digit: true" >> /etc/guacamole/guacamole.properties \
    && echo "postgresql-user-password-prohibit-username: true" >> /etc/guacamole/guacamole.properties

# Link Guacamole home to Tomcat's .guacamole path
RUN rm -rf /usr/share/tomcat8/.guacamole \
    && ln -s /etc/guacamole /usr/share/tomcat8/.guacamole

RUN service postgresql start \
 && su postgres -c "createdb guacamole_db" \
 && cd /opt/incubator-guacamole-client/extensions/guacamole-auth-jdbc/modules/guacamole-auth-jdbc-postgresql/ \
 && cat schema/*.sql | su postgres -c "psql guacamole_db" \
 && su postgres -c "psql -c \"CREATE USER guacamole_user WITH PASSWORD 'PASSWORD';\"" \
 && su postgres -c "psql -c \"GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA public TO guacamole_user;\"" \
 && su postgres -c "psql -c \"GRANT SELECT,USAGE ON ALL SEQUENCES IN SCHEMA public TO guacamole_user;\""

# Re-run ldconfig
RUN ldconfig

# Install a self-signed dhparam for Nginx (this can be slow)
RUN mkdir -p /etc/nginx/ssl && cd /etc/nginx/ssl && openssl dhparam -out dhparams.pem 2048

# Configure environment variable for photoneo
ENV PHOXI_CONTROL_PATH="/opt/PhoXiControl-1.2.26"
ENV PHO_LOCALIZATION_PATH="/opt/PhotoneoSDK/Localization"

# Install required packages for downloading photoneo (including wget)
RUN apt-get update
RUN apt-get install -y ca-certificates
RUN rm -rf /var/lib/apt/lists/*

# Download the PhoXiControl
RUN wget \
     --no-verbose \
     --output-document=/tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu16-STABLE.tar.gz \
     "https://www.photoneo.com/files/dw/dw/pxc/1.2.26/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu16-STABLE.tar.gz"

# Extract the tar.gz archive into /tmp
RUN tar --extract \
     --file=/tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu16-STABLE.tar.gz \
     --directory=/tmp

# Ensure the installer files have execution permissions
RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run
# RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu16-STABLE.run

# Install PhoXiControl to the specified directory
# RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu16-STABLE.run \
RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run \
     --accept "${PHOXI_CONTROL_PATH}"

# Clean up afterwards
RUN rm -rf /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run

# Download PhoLocalization
RUN wget \
     --no-verbose \
     --output-document=/tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run.tar \
     "https://www.photoneo.com/files/dw/dw/localization/1.3.4/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run.tar"

# Extract the tar.gz archive into /tmp
RUN tar --extract \
     --file=/tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run.tar \
     --directory=/tmp

# Ensure the installer files have execution permissions
RUN chmod +x /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# Install PhoXiControl to the specified directory
RUN /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run \
     --accept "${PHO_LOCALIZATION_PATH}"

# Clean up afterwards
RUN rm -rf /tmp/PhotoneoLocalizationSDKInstaller-1.3.4+b9c79c4-Linux-gcc5.5.0.run

# More X related packages to run GUIs:
RUN apt-get update && apt-get install -y \
    ca-certificates \
    xfce4 \
    x11vnc \
    xvfb \
    novnc \
    libxrandr2 \
    libxrender1 \
    libxcursor1 \
    libxi6 \
    libxtst6 \
    libxxf86vm1

# PhoXiControl apparently uses mDNS or DNS-SD for device discovery, 
# requiring an Avahi client and a running Avahi daemon. If the daemon
# isn’t present, PhoXiControl throws std::runtime_error:
RUN apt-get update && apt-get install -y \
    avahi-daemon \
    dbus \
    libavahi-client3

# Expose ports:
# 8080 = Tomcat / Guacamole web
# 4822 = guacd
# 5432 = PostgreSQL
# 80   = Nginx (if you choose to run it)
# 5901 = vnc
# 6901 = noVNC

EXPOSE 8080 4822 5432
EXPOSE 5901 6901 5000

# Need a newer version of cmake to build the Photoneo code
# Install cmake dependencies and clean up
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        apt-transport-https \
        ca-certificates \
        gnupg \
        software-properties-common && \
    rm -rf /var/lib/apt/lists/*

# Import Kitware's signing key and add their repository, then install CMake
RUN wget -qO - https://apt.kitware.com/keys/kitware-archive-latest.asc | apt-key add - && \
    apt-add-repository 'deb https://apt.kitware.com/ubuntu/ xenial main' && \
    apt-get update && \
    apt-get install -y --no-install-recommends cmake && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/local/src

# Clone your C++ repository
RUN git clone https://github.com/sequenceplanner/photoneo_campx.git

# Navigate to the cpp_executables directory
WORKDIR /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables

# Create a 'dev' directory for build artifacts
RUN mkdir -p dev

# Build all four C++ programs
RUN cd dev && \
    for proj in Connect ConnectIPv4 Capture Freerun Stop; do \
        echo "Building $proj..." && \
        mkdir $proj && \
        cd $proj && \
        cmake ../../$proj -DCMAKE_BUILD_TYPE=Release && \
        make -j"$(nproc)" && \
        cd ..; \
    done

# (Optional) Verify build artifacts
RUN ls -al dev/Connect && \
    ls -al dev/ConnectIPv4 && \
    ls -al dev/Capture && \
    ls -al dev/Freerun && \
    ls -al dev/Stop

# Set up Python environment for Flask
WORKDIR /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables

# Create a directory for the Flask API server
RUN mkdir api_server

# Copy the Flask app code into the container
COPY api_server/ api_server/

# Navigate to the Flask app directory
WORKDIR /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables/api_server

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        python3 \
        python3-pip \
        python3-venv

# Create a Python virtual environment and install dependencies with pip 20.3.4
RUN python3 -m venv venv && \
    ./venv/bin/pip install --upgrade "pip==20.3.4" && \
    ./venv/bin/pip install -r requirements.txt

# Make start.sh executable
COPY start.sh /usr/local/src/start.sh
RUN chmod +x /usr/local/src/start.sh

COPY start.sh /start.sh
RUN chmod +x /start.sh

# Final command: run our script that starts all services in one container
CMD ["/start.sh"]



# # ====================================
# # Stage 1: Base Image and OS Dependencies
# # ====================================
# FROM ubuntu:16.04 AS base

# # Set non-interactive mode for apt-get
# ENV DEBIAN_FRONTEND=noninteractive

# # Update OS and install prerequisites
# RUN apt-get update && \
#     apt-get dist-upgrade -y && \
#     apt-get install -y --no-install-recommends \
#         software-properties-common \
#         curl \
#         gnupg \
#         ca-certificates \
#         apt-transport-https \
#         wget \
#         build-essential \
#         autoconf \
#         libtool \
#         maven \
#         pwgen \
#         git \
#         nginx \
#         postgresql-9.5 \
#         postgresql-server-dev-9.5 \
#         postgresql-common \
#         libcairo2-dev \
#         libjpeg-turbo8-dev \
#         libpng12-dev \
#         libossp-uuid-dev \
#         libfreerdp-dev \
#         libpango1.0-dev \
#         libssh2-1-dev \
#         libtelnet-dev \
#         libvncserver-dev \
#         libpulse-dev \
#         libssl-dev \
#         libvorbis-dev \
#         libwebp-dev \
#         ghostscript \
#         haveged \
#         xfce4 \
#         x11vnc \
#         xvfb \
#         novnc \
#         libxrandr2 \
#         libxrender1 \
#         libxcursor1 \
#         libxi6 \
#         libxtst6 \
#         libxxf86vm1 \
#         avahi-daemon \
#         dbus \
#         libavahi-client3 && \
#     rm -rf /var/lib/apt/lists/*

# # Add PPA and install Oracle Java 8
# RUN add-apt-repository ppa:ts.sch.gr/ppa -y && \
#     apt-get update && \
#     echo debconf shared/accepted-oracle-license-v1-1 select true | debconf-set-selections && \
#     echo debconf shared/accepted-oracle-license-v1-1 seen true | debconf-set-selections && \
#     apt-get install -y --no-install-recommends \
#         oracle-java8-installer \
#         oracle-java8-set-default && \
#     rm -rf /var/lib/apt/lists/*

# # # Remove the old Maven if present, and install prerequisites
# RUN apt-get remove -y maven && \
#     apt-get update && \
#     apt-get install -y curl

# # Set environment variables for Maven
# ENV MAVEN_VERSION=3.6.3
# ENV MAVEN_HOME=/opt/maven
# ENV PATH=$MAVEN_HOME/bin:$PATH

# # Install specific Maven version
# RUN wget --no-verbose -O /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz \
#       https://archive.apache.org/dist/maven/maven-3/${MAVEN_VERSION}/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz && \
#     tar -xzf /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz -C /opt && \
#     ln -s /opt/apache-maven-${MAVEN_VERSION} /opt/maven && \
#     ln -s /opt/maven/bin/mvn /usr/bin/mvn && \
#     rm /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz

# # Configure GUACAMOLE_HOME for Tomcat
# RUN echo "" >> /etc/default/tomcat8 && \
#     echo "# GUACAMOLE ENV VARIABLE" >> /etc/default/tomcat8 && \
#     echo "GUACAMOLE_HOME=/etc/guacamole" >> /etc/default/tomcat8

# # Create Guacamole directories & files
# RUN mkdir -p /etc/guacamole/lib /etc/guacamole/extensions && \
#     touch /etc/guacamole/user-mapping.xml

# # ====================================
# # Stage 2: Guacamole Server Setup
# # ====================================
# FROM base AS guacamole-server

# WORKDIR /opt

# # Clone and build Guacamole server
# RUN git clone https://github.com/apache/incubator-guacamole-server.git && \
#     cd incubator-guacamole-server && \
#     autoreconf -fi && \
#     ./configure --with-init-dir=/etc/init.d && \
#     make && \
#     make install && \
#     ldconfig

# # Clone and build Guacamole client (web app)
# RUN git clone --branch 0.9.12-incubating https://github.com/apache/incubator-guacamole-client.git && \
#     cd incubator-guacamole-client && \
#     mvn package && \
#     cp ./guacamole/target/guacamole-0.9.12-incubating.war /var/lib/tomcat8/webapps/guacamole.war && \
#     cp ./extensions/guacamole-auth-jdbc/modules/guacamole-auth-jdbc-postgresql/target/guacamole-auth-jdbc-postgresql-0.9.12-incubating.jar /etc/guacamole/extensions/

# # Install PostgreSQL JDBC driver
# WORKDIR /usr/local/src
# RUN wget -c https://jdbc.postgresql.org/download/postgresql-42.0.0.jar && \
#     cp postgresql-42.0.0.jar /etc/guacamole/lib/

# # Link FreeRDP libraries
# RUN ln -s /usr/local/lib/freerdp/* /usr/lib/x86_64-linux-gnu/freerdp/. || true

# # Configure Guacamole properties
# RUN echo "#api-session-timeout: 60" >> /etc/guacamole/guacamole.properties && \
#     echo "available-languages: en" >> /etc/guacamole/guacamole.properties && \
#     echo "guacd-hostname: localhost" >> /etc/guacamole/guacamole.properties && \
#     echo "guacd-port: 4822" >> /etc/guacamole/guacamole.properties && \
#     echo "#guacd-ssl: true" >> /etc/guacamole/guacamole.properties && \
#     echo "lib-directory: /var/lib/tomcat8/webapps/guacamole/WEB-INF/classes" >> /etc/guacamole/guacamole.properties && \
#     echo "" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-hostname: localhost" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-port: 5432" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-database: guacamole_db" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-username: guacamole_user" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-password: PASSWORD" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-user-password-min-length: 8" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-user-password-require-multiple-case: true" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-user-password-require-symbol: true" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-user-password-require-digit: true" >> /etc/guacamole/guacamole.properties && \
#     echo "postgresql-user-password-prohibit-username: true" >> /etc/guacamole/guacamole.properties

# # Link Guacamole home to Tomcat's .guacamole path
# RUN rm -rf /usr/share/tomcat8/.guacamole && \
#     ln -s /etc/guacamole /usr/share/tomcat8/.guacamole

# # ====================================
# # Stage 3: Photoneo Setup
# # ====================================
# FROM guacamole-server AS photoneo

# # Set environment variables for Photoneo
# ENV PHOXI_CONTROL_PATH="/opt/PhoXiControl-1.2.26"
# ENV PHO_LOCALIZATION_PATH="/opt/PhotoneoSDK/Localization"

# # Install required packages for downloading Photoneo
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # Download and install PhoXiControl
# RUN wget --no-verbose --output-document=/tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run \
#     "https://www.photoneo.com/files/dw/dw/pxc/1.2.26/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run" && \
#     chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run && \
#     /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run --accept "${PHOXI_CONTROL_PATH}" && \
#     rm -rf /tmp/PhotoneoPhoXiControlInstaller-1.2.26-Ubuntu.run

# # Download and install PhoLocalization
# RUN wget --no-verbose --output-document=/tmp/PhotoneoLocalizationSDKInstaller-1.3.4-Linux.run.tar \
#     "https://www.photoneo.com/files/dw/dw/localization/1.3.4/PhotoneoLocalizationSDKInstaller-1.3.4-Linux-gcc5.5.0.run.tar" && \
#     tar --extract --file=/tmp/PhotoneoLocalizationSDKInstaller-1.3.4-Linux.run.tar --directory=/tmp && \
#     chmod +x /tmp/PhotoneoLocalizationSDKInstaller-1.3.4-Linux.run && \
#     /tmp/PhotoneoLocalizationSDKInstaller-1.3.4-Linux.run --accept "${PHO_LOCALIZATION_PATH}" && \
#     rm -rf /tmp/PhotoneoLocalizationSDKInstaller-1.3.4-Linux.run

# # ====================================
# # Stage 4: Build C++ Executables
# # ====================================
# FROM photoneo AS builder

# # Install CMake dependencies and install CMake
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends cmake && \
#     rm -rf /var/lib/apt/lists/*

# # Set working directory
# WORKDIR /usr/local/src

# # Clone your C++ repository
# RUN git clone https://github.com/sequenceplanner/photoneo_campx.git

# # Navigate to the cpp_executables directory
# WORKDIR /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables

# # Create a 'dev' directory for build artifacts
# RUN mkdir -p dev

# # Build all four C++ programs
# RUN cd dev && \
#     for proj in Connect Capture Freerun Stop; do \
#         echo "Building $proj..." && \
#         mkdir $proj && \
#         cd $proj && \
#         cmake ../../$proj -DCMAKE_BUILD_TYPE=Release && \
#         make -j"$(nproc)" && \
#         cd ..; \
#     done

# # (Optional) Verify build artifacts
# RUN ls -al dev/Connect && \
#     ls -al dev/Capture && \
#     ls -al dev/Freerun && \
#     ls -al dev/Stop

# # ====================================
# # Stage 5: Python Environment and Application Code
# # ====================================
# FROM builder AS app

# # Set working directory
# WORKDIR /usr/local/src/photoneo_campx/phoxi_control_interface/cpp_executables

# # Create a directory for the Flask API server
# RUN mkdir api_server

# # Copy the Flask app code into the container
# COPY api_server/ api_server/

# # Install Python dependencies
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends \
#         python3 \
#         python3-pip \
#         python3-venv && \
#     rm -rf /var/lib/apt/lists/*

# # Create a Python virtual environment and install dependencies with pip 20.3.4
# RUN python3 -m venv venv && \
#     ./venv/bin/pip install --upgrade "pip==20.3.4" && \
#     ./venv/bin/pip install -r api_server/requirements.txt

# # ====================================
# # Stage 6: Final Runtime Image
# # ====================================
# FROM app AS final

# # Copy start.sh and make it executable
# COPY start.sh /start.sh
# RUN chmod +x /start.sh

# # Expose ports
# EXPOSE 8080 4822 5432 5901 6901 5000

# # Final command to run the services
# CMD ["/start.sh"]
