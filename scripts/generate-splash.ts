import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  console.log('Generating splashscreen.html...')
}

generateSplashscreen().catch(console.error)
