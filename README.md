# MTBL

Actix Web example using data from [github.com/mledoze/countries](https://github.com/mledoze/countries)

## Usage

### init sled database

```bash
wget -O countries.json https://raw.githubusercontent.com/mledoze/countries/master/countries.json
cargo run --bin builder
```

### server

```bash
cargo run --bin server
```

### web client

[http://127.0.0.1:63333/Germany](http://127.0.0.1:63333/Germany)

### Curl

```bash
curl -H 'Accept: application/json' http://localhost:63333/Germany
curl -H 'Accept: application/yaml' http://localhost:63333/Germany
```
