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
  appName: 'Desktop Shell',
  packageName: 'desktop-shell',
  productName: 'Desktop Shell',
  identifier: 'com.example.desktop-shell',
  description: 'A minimal Tauri React desktop shell.',
  repositoryUrl: 'https://github.com/example/desktop-shell',
  deepLinkScheme: 'desktop-shell',
  updaterEndpoint:
    'https://github.com/example/desktop-shell/releases/latest/download/latest.json',
} satisfies TemplateConfig

export default config