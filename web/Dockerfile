# syntax=docker/dockerfile:1.6

FROM node:20-alpine AS deps
WORKDIR /app
COPY package*.json ./
RUN npm install

FROM node:20-alpine AS build
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:20-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=5173
COPY --from=build /app/package.json ./
COPY --from=build /app/build ./build
COPY --from=deps /app/node_modules ./node_modules
RUN npm prune --omit=dev
EXPOSE 5173
CMD ["node", "build"]
