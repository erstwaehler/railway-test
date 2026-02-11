import { tanstackStart } from '@tanstack/react-start/plugin/vite'
import { defineConfig } from 'vite'
import { nitro } from 'nitro/vite'
import viteReact from '@vitejs/plugin-react'
import viteTsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  resolve: {
    alias: {
      'node:async_hooks': 'async_hooks',
      'node:stream': 'stream',
      'node:stream/web': 'stream/web'
    },
    conditions: ['node', 'import', 'module', 'browser', 'default']
  },
  plugins: [
    tanstackStart(),
    nitro({ preset: 'bun' }),
    viteReact(),
    viteTsconfigPaths({
      configNames: ['tsconfig.json']
    })
  ],
  ssr: {
    noExternal: true,
    external: [
      'node:async_hooks',
      'node:stream',
      'node:stream/web',
      'async_hooks',
      'stream',
      'stream/web'
    ]
  },
  build: {
    rollupOptions: {
      external: [
        "node:async_hooks",
        "node:stream",
        "node:stream/web"
      ]
    }
  }
})
