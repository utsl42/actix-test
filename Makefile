run: frontend/dist/index.html
	cargo run

server:
	cargo build

frontend/graphql_schema.json: server
	cargo run -- schema

frontend/dist/index.html: frontend/graphql_schema.json
	(cd frontend; npm install && npm run dist)

