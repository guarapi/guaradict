## Guaradict

### Descrição do Projeto
O Guaradict é um servidor remoto e um dicionário implementado em Rust, incluindo os drivers de conexão. Na primeira versão, o foco é decidir o tradeoff do teorema CAP, explorando a possibilidade de alta disponibilidade com réplicas, persistência eventual e garantia de consistência.

Alta disponibilidade beirando o em tempo real, com persistência eventual, largamente escalável e com baixa latência.

1. **Alta disponibilidade**: Alta disponibilidade é uma prioridade, com uma arquitetura distribuída com replicação de dados em vários servidores. Isso garantirá que o serviço permaneça disponível mesmo em caso de falha de um servidor.

2. **Quase em tempo real**: Latência otimizada, como o uso de cache em memória para dados frequentemente acessados e o processamento assíncrono de operações de entrada/saída.

3. **Persistência eventual**: A persistência eventual é adequada para garantir a escalabilidade e a baixa latência. Os dados podem ser inicialmente armazenados em memória ou em cache para operações rápidas de leitura e gravação, e então eventualmente persistidos em um armazenamento durável em sistema de arrquivos em disco.

4. **Largamente escalável**: Distribuído e horizontalmente escalável, capacidade de adicionar novos servidores conforme a demanda aumenta, distribuindo a carga de trabalho de forma eficiente entre os servidores.

5. **Baixa latência**: Tempo de processamento otimizado no fluxo de trabalho do servidor, consultas otimizads que reduzem o tempo de resposta da rede e utilização de tecnologias de computação de alto desempenho.


### Mínimo para primeira versão

- Criar o server
- Criar o client
- Salvar um registro em memória
- Recuperar um registro

