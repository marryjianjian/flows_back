#!/bin/bash -ex
# dump old database data and
# import into new database
# when modified table structure
cd $(dirname "$0")

if [ -z "$1" ]
then
    echo "No database supplied"
    exit 1
fi

if [ ! -e "$1" ]
then
    echo "Database not exist"
    exit 2
fi

today="$(date -I)"
OLD_FILE="${today}_$1"
mv "$1" ${OLD_FILE}

sqlite3 "${OLD_FILE}" ".dump access_info" > "${today}_tmp".sql
sqlite3 "$1" < "${today}_tmp".sql

