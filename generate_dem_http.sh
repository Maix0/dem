#!/bin/sh
rm -r ./dem-http
java -jar openapi-generator-cli.jar generate -i http://localhost:8000/api/openapi.json -g rust -o dem-http --additional-properties=packageName=dem-http
echo "Applying Cargo.toml patch"
git apply http.patch
echo "Transforming id to u64"
fastmod -F i32 u64 -d dem-http --accept-all
echo "Transforming body to Vec<u8>"
fastmod -F "request_body: Vec<u64>" "request_body: Vec<u8>" -d dem-http --accept-all
echo "Changing .json to .body"
fastmod -F "local_var_req_builder.json(&request_body)" "local_var_req_builder.body(request_body)" -d dem-http --accept-all
echo "Finished generating dem-http"
