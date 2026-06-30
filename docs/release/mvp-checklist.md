# Tunnel MCP MVP Release Checklist

## 基础能力

- [ ] App name is Tunnel MCP
- [ ] Bundle identifier is com.baicie.tunnel-mcp
- [ ] macOS arm64 installer can launch
- [ ] Windows x64 installer can launch

## 配置

- [ ] OpenAI Key can be saved to secure storage
- [ ] Tunnel ID can be saved
- [ ] OpenAI Key is not visible in plain text after refresh

## tunnel-client

- [ ] Manifest can be fetched
- [ ] sha256 mismatch rejects installation
- [ ] binary can start
- [ ] binary can stop
- [ ] crash updates status

## MCP

- [ ] MCP listens on 127.0.0.1
- [ ] MCP requires local token
- [ ] files/list works under authorized directory
- [ ] files/read works under authorized directory
- [ ] unauthorized path is denied

## Approval

- [ ] files/write creates approval request
- [ ] rejected approval blocks write
- [ ] expired approval blocks write
- [ ] approved request writes only the approved target path

## Logs

- [ ] tunnel events visible
- [ ] MCP events visible
- [ ] permission deny visible
- [ ] approval events visible
- [ ] diagnostic export works
- [ ] no token/key appears in diagnostics
