#!/bin/bash

set -e

DB_NAME=${1:-shereebot}
DB_USER=${2:-mikatpt}
DB_USER_PASS=${3:-testing}

sudo su postgres <<EOF
createdb $DB_NAME
psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_USER_PASS';"
psql -c "GRANT ALL PRIVILEGES on DATABASE $DB_NAME to $DB_USER;"

EOF
