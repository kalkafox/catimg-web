import 'xterm/css/xterm.css'
import './style.css'

import { Terminal } from 'xterm'
import { WebglAddon } from 'xterm-addon-webgl'

import axios from 'axios'

type Spinner = {
  [key: string]: {
    interval: number
    frames: string[]
  }
}

declare global {
  interface Window {
    terminalLoaded: boolean
  }
}

const init = async () => {
  const spinners = await axios.get<Spinner>(
    'https://raw.githubusercontent.com/sindresorhus/cli-spinners/main/spinners.json',
  )

  const spinner = spinners.data.dots3

  let i = 0

  const interval = setInterval(() => {
    terminal.write(spinner.frames[i])
    terminal.write('\b')
    i = ++i % spinner.frames.length
  }, spinner.interval)

  const terminal = new Terminal({
    windowOptions: {
      fullscreenWin: true,
    },
  })

  terminal.loadAddon(new WebglAddon())

  // Get the location path
  const urlParams = new URLSearchParams(window.location.search)
  const w = urlParams.get('w')
  const url = urlParams.get('url')
  const fs = urlParams.get('fs') as boolean | null

  // Get the width and height of the terminal and convert them to cells/columns

  const cells = () => {
    if (fs) {
      return Math.floor(window.innerWidth / 9)
    }
    return Math.floor(window.innerWidth / 10)
  }
  const rows = () => {
    if (fs) {
      return Math.floor(window.innerHeight / 17)
    }
    return Math.floor(window.innerHeight / 20)
  }

  window.onresize = () => {
    // Set the terminal size
    terminal.resize(cells(), rows())
  }

  // Set the terminal size
  terminal.resize(cells(), rows())

  terminal.open(document.querySelector<HTMLDivElement>('#terminal')!)

  if (!url) {
    clearInterval(interval)
    terminal.clear()

    // Use red color for error
    terminal.write('\x1b[31m')
    terminal.writeln('No url provided')

    throw new Error('No url provided')
  }

  const res = await axios.get<string>(
    `https://catimg.kalkafox.dev/catimg?url=${url}${w ? `&w=${w}` : ''}`,
  )

  clearInterval(interval)

  terminal.clear()

  res.data.split('\n').forEach((line) => {
    terminal.writeln(line)
  })

  window.terminalLoaded = true
}

init()
