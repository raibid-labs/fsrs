import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Fusabi',
  description: 'Mini-F# dialect with a Rust VM for embedded scripting',
  base: '/fusabi/',
  srcExclude: ['**/archive/**', '**/audits/**', '**/meta/**', '**/workstreams/**', '**/design/**'],
  ignoreDeadLinks: true,

  markdown: {
    languageAlias: {
      fusabi: 'fsharp'
    }
  },

  head: [
    ['link', { rel: 'icon', href: '/fusabi/favicon.ico' }]
  ],

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/01-overview' },
      { text: 'Cookbook', link: '/cookbook/00-introduction' },
      { text: 'Stdlib', link: '/stdlib/' },
      { text: 'GitHub', link: 'https://github.com/fusabi-lang/fusabi' }
    ],

    sidebar: {
      '/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Overview', link: '/01-overview' },
            { text: 'Language Spec', link: '/02-language-spec' },
            { text: 'VM Design', link: '/03-vm-design' }
          ]
        },
        {
          text: 'Cookbook',
          collapsed: false,
          items: [
            { text: 'Introduction', link: '/cookbook/00-introduction' },
            { text: 'Hello World', link: '/cookbook/01-hello-world' },
            { text: 'Primitives', link: '/cookbook/02-primitives' },
            { text: 'Functions', link: '/cookbook/03-functions' },
            { text: 'Control Flow', link: '/cookbook/04-control-flow' },
            { text: 'Collections', link: '/cookbook/05-collections' },
            { text: 'Modules', link: '/cookbook/06-modules' },
            { text: 'Async', link: '/cookbook/07-async' },
            { text: 'I/O', link: '/cookbook/08-io' }
          ]
        },
        {
          text: 'Standard Library',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/stdlib/' },
            { text: 'List', link: '/stdlib/list' },
            { text: 'Map', link: '/stdlib/map' },
            { text: 'Option', link: '/stdlib/option' },
            { text: 'String', link: '/stdlib/string' }
          ]
        },
        {
          text: 'Reference',
          items: [
            { text: 'Stdlib Reference', link: '/STDLIB_REFERENCE' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/fusabi-lang/fusabi' }
    ],

    search: {
      provider: 'local'
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Fusabi Contributors'
    }
  }
})
