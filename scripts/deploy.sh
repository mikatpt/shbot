#!/bin/bash

set -eou pipefail

DIR=$(dirname "$0")

RED="\e[1;31m"
GREEN="\e[1;32m"
YELLOW="\e[1;33m"
NORM="\e[0m"

function _info() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S") ${GREEN}[INFO]${NORM} ${1}"
}

function _error() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S") ${RED}[ERROR]${NORM} ${1}"
}

function _warn() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S") ${YELLOW}[WARN]${NORM} ${1}"
}

_info "Beginning deployment pipeline"
cd "$DIR"/..


_info "Building docker image"
docker-compose build

_info "Cleaning up old images"
docker image prune -f

_info "Authenticating with docker"
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin ***REMOVED***


_info "Pushing image to docker"
docker tag shbot_api ***REMOVED***/shbot_api:latest
docker push ***REMOVED***/shbot_api:latest
