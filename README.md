# actionvk - хранилище "ключ-значение".

Хранилище предназначено для хранения и звлечения
последовательности байтов произвольной длины.
Каждая последовательность состоит из дух частей: ключа и значения.

## Как комплироать:

1. Установить Rust https://www.rust-lang.org/tools/install.

2. Клонировать репозиторий.

3. Зайти в директорию проекта.
```
cd actionvk/
```
4. Компилирование проекта.
```
cargo build
```
5. Запуск тестов.
```
cargo test --package actionkv --lib -- test --test-threads=1
```

# Как использовать:

## insert запрос
 ```
cargo run --bin akv_disk test insert apple 100
 ```

## get запрос
```
cargo run --bin akv_disk test get apple
```
 Результат запроса: "100"

## find запрос
```
cargo run --bin akv_disk test find apple
```
Результат запроса: "100"

## update запрос
```
cargo run --bin akv_disk test update apple 200
```

## delete запрос
```
cargo run --bin akv_disk test delete apple
```
После удаления get запрос вернет пустую строку "".