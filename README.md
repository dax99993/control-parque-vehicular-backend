# Backend para servicio de control parque vehicular.


## Requisitos
- Docker
- Toolchain de Rust

## Instalacion

Instalar toolchain de Rust
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clonar el repositorio
```sh
git clone https://github.com/dax99993/control-parque-vehicular-backend
cd control-parque-vehicular-backend 
```

### Ejecutar los scripts
Esto creara la base de datos en Postgres, un Cache para blacklist de tokens en Redis y un servidor para correos con docker y con la configuracion para desarrollar en local,
```sh
./scripts/init_db.sh
./scripts/redis_db.sh
./scripts/init_smtp.sh
```

### Configuracion
Si se desea cambiar la configuracion se modifican los archivos en la carpeta configuration,
como puede ser para cambiar el puerto del backend en base.yml.

### Ejecucion
Finalmente para ejecutar el backend se debe estar en la carpeta principal del proyecto donde se encuentra el archivo Cargo.toml
```sh
cargo run
```
o si se tiene instalado jq para facilitar la lectura de la temeletria
```sh
cargo run | jq
```
