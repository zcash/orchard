import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';

const FORK_OWNER = 'dannywillems';
const FORK_REPO = 'orchard';
const FORK_BRANCH = 'onboarding';
const FORK_URL = `https://github.com/${FORK_OWNER}/${FORK_REPO}`;
const UPSTREAM_URL = 'https://github.com/zcash/orchard';

const config: Config = {
  title: 'Orchard onboarding',
  tagline: 'A graduate-level reading course on the zcash/orchard crate',
  favicon: 'img/favicon.ico',

  url: `https://${FORK_OWNER}.github.io`,
  baseUrl: `/${FORK_REPO}/`,

  organizationName: FORK_OWNER,
  projectName: FORK_REPO,

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  markdown: {
    format: 'detect',
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: '/',
          editUrl: `${FORK_URL}/edit/${FORK_BRANCH}/onboarding/`,
          remarkPlugins: [remarkMath],
          rehypePlugins: [rehypeKatex],
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  stylesheets: [
    {
      href: 'https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css',
      type: 'text/css',
      integrity:
        'sha384-nB0miv6/jRmo5EGIE6RDQE0etf4GvjBR1bkf4pcUk2TprLGa0k7/rJkRnCu6WSt6',
      crossorigin: 'anonymous',
    },
  ],

  themes: [
    [
      '@easyops-cn/docusaurus-search-local',
      {
        hashed: true,
        indexBlog: false,
        indexDocs: true,
        indexPages: true,
        language: ['en'],
        highlightSearchTermsOnTargetPage: true,
      },
    ],
    'docusaurus-theme-github-codeblock',
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    announcementBar: {
      id: 'ai-generated-disclaimer',
      content:
        'This site is automatically generated using Claude Code. Errors may have been introduced. The code is the law, always refer to the source in the zcash/orchard crate.',
      backgroundColor: '#fef3c7',
      textColor: '#78350f',
      isCloseable: false,
    },
    navbar: {
      title: 'Orchard onboarding',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Course',
        },
        {
          href: `${FORK_URL}/tree/${FORK_BRANCH}`,
          label: 'GitHub (fork)',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Source',
          items: [
            {
              label: 'Fork (this site)',
              href: `${FORK_URL}/tree/${FORK_BRANCH}`,
            },
            {
              label: 'Upstream zcash/orchard',
              href: UPSTREAM_URL,
            },
          ],
        },
        {
          title: 'References',
          items: [
            {
              label: 'Zcash Protocol Specification',
              href: 'https://zips.z.cash/protocol/protocol.pdf',
            },
            {
              label: 'ZIPs index',
              href: 'https://zips.z.cash/',
            },
            {
              label: 'Halo 2 Book',
              href: 'https://zcash.github.io/halo2/',
            },
          ],
        },
      ],
      copyright: 'Orchard onboarding course, ASCII only, neutral voice.',
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'bash', 'toml', 'json', 'yaml'],
    },
    codeblock: {
      showGithubLink: true,
      githubLinkLabel: 'View on GitHub',
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
