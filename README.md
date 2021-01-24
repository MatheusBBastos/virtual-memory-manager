# Gerenciador de Memória Virtual

* Livro: Operating System Concepts - 9th Edition (Abraham Silberschatz)
* Discente: Matheus Barbiero Bastos
* Docente: Fabio Sakuray

## Informações

* TLB e substituição de páginas foram implementados com política FIFO.
* Por padrão, o espaço de endereçamento físico está configurado para ter 128 frames. Essa configuração está na constante `NUM_FRAMES`, presente na linha 8 do arquivo `src/main.rs`.

## Uso

Antes de tudo, deve haver um arquivo chamado `BACKING_STORE.bin` no diretório de execução do programa. Ele tem um formato binário, com 65536 bytes (256 páginas de 256 números de 1 byte).

O arquivo `addresses.txt` (pode ter outro nome) deve ser passado como primeiro argumento ao programa, e contém um endereço de 0 a 65535 por linha (no formato decimal).

Para compilar e executar o programa:
```shell
cargo run addresses.txt
```

Também é possível compilar e executar depois:
```shell
rustc src/main.rs -o vmm
./vmm addresses.txt
```



