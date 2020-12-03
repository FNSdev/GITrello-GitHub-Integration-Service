# GITrello

[![Build Status](https://gitrello.me/jenkins/buildStatus/icon?job=gitrello-github-integration-service)](https://gitrello.me/jenkins/job/gitrello/)

## How to run locally

### Tested on

* Linux Mint 19.3
* rustup 1.22.1
* cargo 1.47.0
* Docker 19.03.14
* Docker-Compose 1.26.0

### 1. Install required libraries

```
$ sudo apt install ca-certificates libssl-dev libpq-dev
```

### 2. Clone repository

```
$ git clone https://github.com/FNSdev/gitrello-github-integration-service.git
$ cd gitrello-github-integration-service
```

### 3. Install dependencies & build project

Install diesel-cli

```
$ cargo install diesel_cli
```

Install project dependencies & build project. It might take some time to compile all dependencies.

```
$ cargo build
```

#### 4. Run database using Docker-Compose

```
$ docker-compose up
```

This will automatically create Database and User.

#### 5. Apply migrations

```
$ DATABASE_URL=postgres://gitrello_github_integration_service:admin@127.0.0.1:5432/gitrello_github_integration_service diesel migration run
```

#### 6. Run server

```
$ cargo run
```
