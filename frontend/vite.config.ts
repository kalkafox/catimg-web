import { defineConfig } from 'vite'

export default defineConfig({
  // ROllup options
  build: {
    rollupOptions: {
      output: {
        // Provide global variables to use in the UMD build
        minifyInternalExports: true,
        manualChunks: {
          axios: ['axios'],
          xterm: ['xterm'],
          'xterm-addon-webgl': ['xterm-addon-webgl'],
        },
      },
    },
  },
})
