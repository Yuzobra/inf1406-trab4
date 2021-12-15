#!/bin/bash
set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

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
if ./bin/run_client.sh SEARCH CHAVE 10 0 10 3; then # Expect server 3 to respond
    echo -e "${GREEN}TEST 1... PASSED${NC}\n"
else
    all_tests_successfull=1
    echo -e "${RED}TEST 1... FAILED${NC}\n"
fi

# TEST 2
# Kill server and add content to servers, expect the substitute to return
./bin/run_client.sh DIE "" 3 0 -1 -1 # Kill server 3
./bin/run_client.sh INSERT CHAVE 20 0 -1 -1 # Change value on "CHAVE" 
if ./bin/run_client.sh SEARCH CHAVE -1 0 20 0; then # Expect server #0 to respond
    echo -e "${GREEN}TEST 2... PASSED${NC}\n"
else
    all_tests_successfull=1
    echo -e "${RED}TEST 2... FAILED${NC}\n"
fi

# TEST 3
# Wait server #3 to restore and respond SEARCH 
sleep 25
if ./bin/run_client.sh SEARCH CHAVE -1 0 20 3; then # Expect server #3 to respond with new value
    echo -e "${GREEN}TEST 3... PASSED${NC}\n"
else
    all_tests_successfull=1
    echo -e "${RED}TEST 3... FAILED${NC}\n"
fi


if [ "$all_tests_successfull" -eq "0" ]; then
    echo -e "${GREEN}TESTS FINISHED: Everything passed successfully${NC}"
else 
    echo -e "${RED}There were failing tests, check logs${NC}"
fi

sleep 5
kill -TERM -$monitor_gid && docker rm -f $(docker ps -aq)