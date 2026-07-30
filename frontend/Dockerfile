# --- build stage ---
FROM node:22-slim AS builder
WORKDIR /app
# npm ci (e não npm install) para instalar exatamente as versões do lock —
# as mesmas que o CI validou. Exige package-lock.json presente.
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# --- runtime stage ---
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
