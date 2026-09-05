VERSION 5.00
Begin VB.Form Form1 
   Caption         =   "Form1"
   ClientHeight    =   3135
   ClientLeft      =   60
   ClientTop       =   405
   ClientWidth     =   4680
   LinkTopic       =   "Form1"
   ScaleHeight     =   3135
   ScaleWidth      =   4680
   StartUpPosition =   3  'Windows Default
End
Attribute VB_Name = "Form1"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Option Explicit

Private Declare Function IniciarServicoTcpViaWS Lib "tcp_over_ws.dll" Alias "spawn_tcp_over_ws" (ByVal remote_ws_service_url As String, ByVal endereco_escutar As String, Optional ByVal timeout As Long = 30000) As Boolean

Private Sub Form_Load()
MsgBox IniciarServicoTcpViaWS("wss://fb-getech.app-pratico.com.br", "127.0.0.1:19258")
End Sub
