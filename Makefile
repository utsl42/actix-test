build: frontend/graphql_schema.json
	(cd frontend; npm run dist)

prep:
	(cd frontend; npm install)

frontend/graphql_schema.json:
	./server schema

