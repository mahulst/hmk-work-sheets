FROM codesimple/elm:0.19 AS builder

COPY . /home/app

WORKDIR /home/app

RUN npm install
RUN npm run build


FROM nginx:stable
COPY ./nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /home/app/dhparam-2048.pem /etc/ssl/certs/dhparam-2048.pem
COPY --from=builder /home/app/build /var/www
