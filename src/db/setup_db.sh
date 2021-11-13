#!/bin/bash -ex
cd $(dirname "$0")
sqlite3 ./access.db < init.sql
# sqlite3 -csv ./access.db ".import access_test_data.csv access_info"

