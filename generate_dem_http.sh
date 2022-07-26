java -jar openapi-generator-cli.jar generate -i http://localhost:8000/api/openapi.json -g rust -o dem-http --additional-properties=packageName=dem-http
