```basic
'remote_ws_service_url = "ws://127.0.0.1:80"
'endereco_escutar = "127.0.0.1:19258"

'retorna falso caso os argumentos sejam inválidos
Private Declare Function IniciarServicoTcpViaWSTeste Lib "tcp_over_ws.dll" Alias "spawn_tcp_over_ws_test" (ByVal remote_ws_service_url As String, ByVal endereco_escutar As String) As Boolean

'cria uma thread que vai servir o servidor tcp local que conecta ao serviço ws_to_tcp via ws
'retorna falso caso os argumentos sejam inválidos ou caso não seja possível escutar na porta
Private Declare Function IniciarServicoTcpViaWS Lib "tcp_over_ws.dll" Alias "spawn_tcp_over_ws" (ByVal remote_ws_service_url As String, ByVal endereco_escutar As String, Optional ByVal timeout As Long = 30000) As Boolean
```

o exe é um serviço do windows, que lê a configuração de `config.toml`, rode ele para ele criar esse arquivo

ele pode ser instalado rodando `ws_to_tcp.exe install` no terminal, `rode ws_to_tcp.exe --help` para ver mais opções
