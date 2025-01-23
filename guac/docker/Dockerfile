#
# Dockerfile for Apache Guacamole (server + client) on Ubuntu 16.04
# with PostgreSQL running in same container (NOT recommended for production).
#

FROM ubuntu:16.04

# Noninteractive install
ENV DEBIAN_FRONTEND=noninteractive

# 1) Update OS and install prerequisites
RUN apt-get update \
    && apt-get dist-upgrade -y \
    && apt-get install -y software-properties-common

# 2) Install packages
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

# 3) Create Guacamole directories & files
RUN mkdir -p /etc/guacamole \
    && mkdir -p /etc/guacamole/lib \
    && mkdir -p /etc/guacamole/extensions \
    && touch /etc/guacamole/user-mapping.xml

# 4) Configure GUACAMOLE_HOME for Tomcat
RUN echo "" >> /etc/default/tomcat8 \
    && echo "# GUACAMOLE ENV VARIABLE" >> /etc/default/tomcat8 \
    && echo "GUACAMOLE_HOME=/etc/guacamole" >> /etc/default/tomcat8

# 5) Build and install Guacamole server
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
    apt-get install -y wget curl

# Install Maven 3.6.3 manually
ENV MAVEN_VERSION=3.6.3
RUN wget --no-verbose -O /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz \
      https://archive.apache.org/dist/maven/maven-3/3.6.3/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz \
    && tar -xzf /tmp/apache-maven-${MAVEN_VERSION}-bin.tar.gz -C /opt \
    && ln -s /opt/apache-maven-${MAVEN_VERSION} /opt/maven \
    && ln -s /opt/maven/bin/mvn /usr/bin/mvn

ENV MAVEN_HOME=/opt/maven
ENV PATH=$MAVEN_HOME/bin:$PATH

# 6) Build and install Guacamole client (web app)
WORKDIR /opt
RUN git clone --branch 0.9.12-incubating https://github.com/apache/incubator-guacamole-client.git \
    && cd incubator-guacamole-client \
    && mvn package \
    && cp ./guacamole/target/guacamole-0.9.12-incubating.war /var/lib/tomcat8/webapps/guacamole.war \
    && cp ./extensions/guacamole-auth-jdbc/modules/guacamole-auth-jdbc-postgresql/target/guacamole-auth-jdbc-postgresql-0.9.12-incubating.jar \
          /etc/guacamole/extensions/

# 7) Install PostgreSQL JDBC driver
WORKDIR /usr/local/src
RUN wget -c https://jdbc.postgresql.org/download/postgresql-42.0.0.jar \
    && cp postgresql-42.0.0.jar /etc/guacamole/lib/

# 8) Link FreeRDP libraries (as in your script)
RUN ln -s /usr/local/lib/freerdp/* /usr/lib/x86_64-linux-gnu/freerdp/. || true

# 9) Generate a random password example (you can remove or adapt)
RUN pwgen -n 14

# 10) Configure Guacamole properties
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

# 11) Link Guacamole home to Tomcat's .guacamole path
RUN rm -rf /usr/share/tomcat8/.guacamole \
    && ln -s /etc/guacamole /usr/share/tomcat8/.guacamole

# Works until here

RUN service postgresql start \
 && su postgres -c "createdb guacamole_db" \
 && cd /opt/incubator-guacamole-client/extensions/guacamole-auth-jdbc/modules/guacamole-auth-jdbc-postgresql/ \
 && cat schema/*.sql | su postgres -c "psql guacamole_db" \
 && su postgres -c "psql -c \"CREATE USER guacamole_user WITH PASSWORD 'PASSWORD';\"" \
 && su postgres -c "psql -c \"GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA public TO guacamole_user;\"" \
 && su postgres -c "psql -c \"GRANT SELECT,USAGE ON ALL SEQUENCES IN SCHEMA public TO guacamole_user;\""

# 13) Re-run ldconfig
RUN ldconfig

# 14) Install a self-signed dhparam for Nginx (this can be slow).
RUN mkdir -p /etc/nginx/ssl && cd /etc/nginx/ssl && openssl dhparam -out dhparams.pem 2048

# 15) Expose typical ports:
    # 8080 = Tomcat / Guacamole web
    # 4822 = guacd
    # 5432 = PostgreSQL
    # 80   = Nginx (if you choose to run it)
EXPOSE 8080 4822 5432 

# 16) Configure environment variable for photoneo
ENV PHOXI_CONTROL_PATH="/opt/PhoXiControl-1.2.35"

# 17) Install required packages for downloading photoneo (including wget)
RUN apt-get update
RUN apt-get install -y wget ca-certificates
RUN rm -rf /var/lib/apt/lists/*

# 18) Download the tar.gz from the photoneo's link
RUN wget \
     --no-verbose \
     --output-document=/tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.tar.gz \
     "https://www.photoneo.com/files/dw/dw/pxc/1.2.35/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.tar.gz"

# 19) Extract the tar.gz archive into /tmp
RUN tar --extract \
     --file=/tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.tar.gz \
     --directory=/tmp

# 20) Ensure the installer files have execution permissions
RUN chmod +x /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

# 21) Install PhoXiControl to the specified directory
RUN /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run \
     --accept "${PHOXI_CONTROL_PATH}"

# 22) Clean up afterwards
RUN rm -rf /tmp/PhotoneoPhoXiControlInstaller-1.2.35-Ubuntu16-STABLE.run

CMD ["PhoXiControl"]