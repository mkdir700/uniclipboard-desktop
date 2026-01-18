import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { glob } from 'glob'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  console.log('Generating splashscreen.html...')

  // 1. 读取并合并所有主题 CSS
  const themeFiles = await glob('src/styles/themes/*.css', { cwd: root })
  console.log(`Found ${themeFiles.length} theme files:`, themeFiles)

  const themesCss = themeFiles
    .map(file => {
      const content = fs.readFileSync(path.join(root, file), 'utf-8')
      // 移除 @layer base 包装和闭合括号
      return content.replace(/@layer base\s*{?\s*/g, '').replace(/}\s*$/, '')
    })
    .join('\n')

  console.log(`Merged CSS length: ${themesCss.length} bytes`)

  // 2. 输出到临时文件验证
  fs.writeFileSync(path.join(root, 'public/themes-debug.css'), themesCss)
  console.log('✓ Wrote public/themes-debug.css for verification')
}

generateSplashscreen().catch(console.error)
