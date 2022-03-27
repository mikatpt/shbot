#!/bin/bash

set -eou pipefail

# Just poll the first instance, clean up later if we care.
INSTANCE_ID=$(terraform show -json | jq '.values.outputs.instance_ids.value[0]' | sed 's/"//g')

function _poll() {
    aws ec2 describe-instance-status | jq ".InstanceStatuses[] |
        select(.InstanceId==\"${INSTANCE_ID}\").InstanceStatus.Status" | sed 's/"//g'
}

RES=$(_poll)

echo "Polling status for $INSTANCE_ID..."
while :
do
    if [ $(_poll) == "initializing" ] 
    then
        echo "still initializing"
    else
        echo "Ok"
        break
    fi
    sleep 5
done
