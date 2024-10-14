import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Pumpkin",
  description:
    "Empowering everyone to host fast and efficient Minecraft servers",
  lang: "en-US",
  base: "/Pumpkin/",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    search: {
      provider: "local",
    },
    sidebar: [
      {
        text: "About",
        items: [
          { text: "Introduction", link: "/about/introduction" },
          { text: "Quick Start", link: "/about/quick-start" },
        ],
      },
      {
        text: "Developers",
        items: [
          {
            text: "Contributing",
            link: "https://github.com/Snowiiii/Pumpkin/blob/master/CONTRIBUTING.md",
          },
          { text: "Introduction", link: "/developer/introduction" },
          { text: "Networking", link: "/developer/networking" },
          { text: "Authentication", link: "/developer/authentication" },
        ],
      },
      
      {
        text: "Configuration",
        items: [
          { text: "Introduction", link: "/config/introduction" },
          { text: "Basic", link: "/config/basic" },
          { text: "Advanced", link: "/config/advanced" },
        ],
      },
      {
        text: "Troubleshooting",
        items: [
          { text: "Common Issues", link: "/troubleshooting/common_issues.md" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/Snowiiii/Pumpkin" },
      { icon: "discord", link: "https://discord.gg/RNm224ZsDq" },
    ],

    logo: "/assets/icon.png",
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2024-present Aleksandr Medvedev",
    },
    editLink: {
      pattern: "https://github.com/Snowiiii/Pumpkin/blob/master/docs/:path",
      text: "Edit this page on GitHub",
    },
    lastUpdated: {
      text: "Updated at",
      formatOptions: {
        dateStyle: "medium",
        timeStyle: "medium",
      },
    },
    outline: "deep"
  },
  head: [["link", { rel: "icon", href: "/assets/favicon.ico" }]],
});
