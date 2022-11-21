build:
    docker build -t heosd .

run: build
    docker run -d --env-file .env --expose 8080 -p 8080:8080 heosd /heos-axum
