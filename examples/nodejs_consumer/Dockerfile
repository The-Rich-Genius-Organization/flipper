FROM node:20

WORKDIR /usr/src/app

RUN apt update && apt update -y  && \
	apt install net-tools dnsutils -y

COPY package*.json ./
RUN npm install
RUN npm install -g nodemon

COPY . .

CMD [ "nodemon", "main.js" ]
