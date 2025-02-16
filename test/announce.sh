#! /usr/bin/env bash

curl --insecure -d '{"alias":"TEST","version":"2.0", "fingerprint": "foo", "port": 53317, "protocol": "https", "download": true}' -H 'Content-Type: application/json' -X POST https://${1}:53317/api/localsend/v2/register

# Add newline
echo
