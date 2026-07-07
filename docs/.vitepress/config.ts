import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'model2vec-serve',
  description: 'Lightweight OpenAI and TEI compatible embeddings server for model2vec models',
  base: '/model2vec-serve/',
  cleanUrls: true,
  lastUpdated: true,

  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/introduction' },
      { text: 'API', link: '/api/openai' },
      { text: 'Deployment', link: '/deployment/docker' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Introduction', link: '/introduction' },
          { text: 'Getting Started', link: '/getting-started' },
          { text: 'Configuration', link: '/configuration' },
          { text: 'Architecture', link: '/architecture' },
          { text: 'Development', link: '/development' },
        ],
      },
      {
        text: 'API Reference',
        items: [
          { text: 'OpenAI Embeddings', link: '/api/openai' },
          { text: 'TEI Compatibility', link: '/api/tei' },
          { text: 'Health & Metrics', link: '/api/health-and-metrics' },
          { text: 'Errors', link: '/api/errors' },
        ],
      },
      {
        text: 'Deployment',
        items: [
          { text: 'Docker', link: '/deployment/docker' },
          { text: 'Helm', link: '/deployment/helm' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/freinold/model2vec-serve' },
    ],

    footer: {
      message: 'Released under the MIT License.',
    },

    editLink: {
      pattern: 'https://github.com/freinold/model2vec-serve/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },

    search: {
      provider: 'local',
    },
  },
});
