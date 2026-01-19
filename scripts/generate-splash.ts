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
  const themesCss = themeFiles
    .map(file => {
      const content = fs.readFileSync(path.join(root, file), 'utf-8')
      // 移除 @layer base 包装和闭合括号
      return content.replace(/@layer base\s*{?\s*/g, '').replace(/}\s*$/, '')
    })
    .join('\n')

  // 2. 读取动画 CSS
  const animationsCss = fs.readFileSync(path.join(root, 'src/splashscreen/animations.css'), 'utf-8')

  // 3. 读取 HTML 模板
  const template = fs.readFileSync(path.join(root, 'src/splashscreen/template.html'), 'utf-8')

  // 4. 注入内容
  const html = template
    .replace('/* ANIMATIONS_CSS */', animationsCss)
    .replace('/* THEMES_CSS */', themesCss)

  // 5. 写入输出
  fs.writeFileSync(path.join(root, 'public/splashscreen.html'), html)

  console.log('✓ Generated public/splashscreen.html')
}

generateSplashscreen().catch(console.error)
