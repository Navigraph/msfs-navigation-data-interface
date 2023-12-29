#!/bin/bash

rm -rf test_out
mkdir test_out

npm install
npm run jest
