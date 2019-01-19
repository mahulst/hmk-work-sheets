FROM codesimple/elm:0.19 AS builder
RUN mkdir -p /home/app/
COPY ./package.json /home/app
WORKDIR /home/app
RUN npm install

COPY . /home/app

RUN npm run build


FROM nginx:stable
COPY ./nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /home/app/build /var/www
