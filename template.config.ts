export interface TemplateConfig {
  appName: string
  packageName: string
  productName: string
  identifier: string
  description: string
  repositoryUrl: string
  deepLinkScheme: string
  updaterEndpoint: string
}

const config = {
  appName: 'Tunnel MCP',
  packageName: 'tunnel-mcp',
  productName: 'Tunnel MCP',
  identifier: 'com.baicie.tunnel-mcp',
  description: 'Local MCP gateway desktop client.',
  repositoryUrl: 'https://github.com/baicie/tunnel-mcp',
  deepLinkScheme: 'tunnel-mcp',
  updaterEndpoint:
    'https://github.com/baicie/tunnel-mcp/releases/latest/download/latest.json',
} satisfies TemplateConfig

export default config