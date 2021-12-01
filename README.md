### Alunos: 
Fernando Santos 
Yuri Zoel

### Instruções

Para rodar os servidores: ./bin/run_all_servers.sh

Para rodar somente um servidor: ./bin/run_server.sh <SERVER_NUM> <TOTAL_SERVER_COUNT>

Para rodar o cliente: ./bin/run_client.sh <INSERT ou SEARCH> <SERVER_NODE_KEY> <CONNECTION_ID> <VALUE> <SERVER_CONTACT_NODE_NUMBER> <CLIENT_RETURN_IP> <NUMBER_OF_CLIENT_THREADS> <NUMBER_OF_REQUESTS_PER_THREAD>

NOTA: O valor <NUMBER_OF_REQUESTS_PER_THREAD> não pode ser maior que 99 (devido à porta utilizada pela threads).

NOTA 2: Se os parâmetros <NUMBER_OF_CLIENT_THREADS> <NUMBER_OF_REQUESTS_PER_THREAD> forem passados para o cliente, haverá alteração em alguns dos valores passados para o servidor. Isto serve para garantir que todas as threads poderão se conectar ao servidor, e que irão passar valores aonde seja possível validar que a operação está correta.

NOTA 3: Durante a busca, caso o <VALUE> seja diferente de -1, o cliente irá validar se o valor recebido coincide com o passado.