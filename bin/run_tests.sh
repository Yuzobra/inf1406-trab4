#!/bin/bash
set -e

# SETUP
docker-compose up -d
setsid sh -c "./bin/run_monitor.sh 4 & wait" &
monitor_gid=$!
sleep 5
trap "kill -TERM -$monitor_gid && docker rm -f $(docker ps -aq)" SIGINT
all_tests_successfull=0

# TEST 1
# Set and search key on server 3
./bin/run_client.sh INSERT CHAVE 10 0 -1 -1
if ./bin/run_client.sh SEARCH CHAVE 10 0 10 3; then
    echo "TEST 1... PASSED"
else
    all_tests_successfull=1
    echo "TEST 1... FAILED"
fi

if [ "$all_tests_successfull" -eq "0" ]; then
    echo "TESTS FINISHED: Everything passed successfully"
else 
    echo "There were failing tests, check logs"
fi

# TEST 2
# Kill server and wait it recovery
./bin/run_client.sh DIE "" 3 0 -1 -1

exec 3>&1
exec 2>&1 | tee file.txt

while :
do
    echo "Echoing output"
    cat file.txt
    sleep 1
done



sleep 5
kill -TERM -$monitor_gid && docker rm -f $(docker ps -aq)