FROM nginx:1.14
RUN apt update && apt install curl -y
COPY ./nginx.conf /etc/nginx/nginx.conf
COPY ./proxy.conf /etc/nginx/conf.d/default.conf
COPY ./dhparam-2048.pem /etc/ssl/certs/dhparam-2048.pem
