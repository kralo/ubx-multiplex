#!/bin/bash
COUNTER=0
while [  $COUNTER -lt 10000 ]; do
   echo \$GNTXT,01,01,02,S2: counter ist at $COUNTER*BB
   let COUNTER=COUNTER+1
   sleep $((RANDOM%3))
done
