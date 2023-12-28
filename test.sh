#!/bin/bash

HOST="127.0.0.1"
PORT=6969

send_request() {
	echo -n "hi miene fruende!!" | telnet $HOST $PORT  
}

for ((i=1; i<=100; i++)); do
	send_request &
done

wait
