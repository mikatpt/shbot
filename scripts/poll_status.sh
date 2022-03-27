#!/bin/bash

set -eou pipefail

RED="\e[1;31m"
GREEN="\e[1;32m"
YELLOW="\e[1;33m"
GRAY="\e[1;90m"
NORM="\e[0m"

function _info() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S")  ${GREEN}[INFO]${GRAY} ${1}${NORM}"
}

function _warn() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S")  ${YELLOW}[WARN]${GRAY} ${1}${NORM}"
}

function _error() {
    echo -e "$(date +"%Y-%m-%d %H:%M:%S")  ${RED}[ERROR]${GRAY} ${1}${NORM}"
}

INSTANCES=$(
    aws ec2 describe-instance-status | jq -r '.InstanceStatuses[].InstanceId | @sh' | sed "s/'//g"
)

function _poll_ec2() {
    aws ec2 describe-instance-status | jq -r ".InstanceStatuses[] |
        select(.InstanceId==\"${1}\").InstanceStatus.Status"
}

TIMEOUT=5
# Poll given instance for health status every $TIMEOUT seconds
function _poll_id() {
    while :
    do
        RES=$(_poll_ec2 $1)
        if [ $RES == "initializing" ] 
        then
            _info "still initializing"
        elif [ $RES == "ok" ]
        then
            _info "Instance is healthy!"
            break
        else
            _error "Instance did not become healthy; exiting"
            exit 1
        fi
        sleep $TIMEOUT
    done

}

# Poll all instances for health
for ID in "${INSTANCES[@]}"
do
    :
    _info "Polling health status for $ID..."
    _poll_id $ID
done
