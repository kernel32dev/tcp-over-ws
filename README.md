```basic
'remote_ws_service_url = "ws://127.0.0.1:80"
'endereco_escutar = "127.0.0.1:19258"
'chame essa função para iniciar o serviço, não tem problema chamar multiplas vezes e não precisa chamar algo para parar o serviço
'retorna falso caso os argumentos sejam inválidos
Private Declare Function IniciarServicoTcpViaWS Lib "tcp_over_ws.dll" Alias "spawn_tcp_over_ws" (ByVal remote_ws_service_url As String, ByVal endereco_escutar As String, Optional ByVal timeout As Long = 30000) As Boolean
```