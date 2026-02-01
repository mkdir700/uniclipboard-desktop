#!/usr/bin/env node

/* global process */

/**
 * Version Bump Script
 *
 * Usage:
 *   node scripts/bump-version.js --type patch --channel alpha
 *   node scripts/bump-version.js --type minor --channel stable
 *   node scripts/bump-version.js --type major --channel beta
 *
 * Options:
 *   --type <patch|minor|major>  Version bump type (required)
 *   --channel <stable|alpha|beta|rc>  Release channel (default: stable)
 *   --dry-run  Show what would be changed without writing
 */

import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

// Parse command line arguments
function parseArgs() {
  const args = process.argv.slice(2)
  const options = {
    type: null,
    channel: 'stable',
    dryRun: false,
  }

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--type' && args[i + 1]) {
      options.type = args[i + 1]
      i++
    } else if (args[i] === '--channel' && args[i + 1]) {
      options.channel = args[i + 1]
      i++
    } else if (args[i] === '--dry-run') {
      options.dryRun = true
    }
  }

  return options
}

// Validate semver format
function parseSemver(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z]+)\.(\d+))?$/)
  if (!match) {
    throw new Error(`Invalid semver format: ${version}`)
  }

  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] || null,
    prereleaseVersion: match[5] ? parseInt(match[5], 10) : null,
  }
}

// Format version object back to string
function formatVersion(ver) {
  let version = `${ver.major}.${ver.minor}.${ver.patch}`
  if (ver.prerelease) {
    version += `-${ver.prerelease}.${ver.prereleaseVersion}`
  }
  return version
}

// Calculate next version
function bumpVersion(currentVersion, type, channel) {
  const ver = parseSemver(currentVersion)
  const isStable = channel === 'stable'

  // If bumping from prerelease to stable, just remove prerelease tag
  if (isStable && ver.prerelease) {
    ver.prerelease = null
    ver.prereleaseVersion = null
    return formatVersion(ver)
  }

  // If same channel as current prerelease, only increment prerelease version
  if (!isStable && ver.prerelease === channel) {
    ver.prereleaseVersion += 1
    return formatVersion(ver)
  }

  // Bump version numbers (new channel or stable release)
  if (type === 'major') {
    ver.major += 1
    ver.minor = 0
    ver.patch = 0
  } else if (type === 'minor') {
    ver.minor += 1
    ver.patch = 0
  } else if (type === 'patch') {
    ver.patch += 1
  } else {
    throw new Error(`Invalid bump type: ${type}. Must be patch, minor, or major.`)
  }

  // Handle prerelease channels
  if (!isStable) {
    ver.prerelease = channel
    ver.prereleaseVersion = 1
  } else {
    ver.prerelease = null
    ver.prereleaseVersion = null
  }

  return formatVersion(ver)
}

// Update package.json
function updatePackageJson(newVersion, dryRun) {
  const pkgPath = path.join(process.cwd(), 'package.json')
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'))
  const oldVersion = pkg.version

  pkg.version = newVersion

  if (!dryRun) {
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8')
  }

  return { path: pkgPath, old: oldVersion, new: newVersion }
}

// Update tauri.conf.json
function updateTauriConfig(newVersion, dryRun) {
  const configPath = path.join(process.cwd(), 'src-tauri', 'tauri.conf.json')
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'))
  const oldVersion = config.version

  config.version = newVersion

  if (!dryRun) {
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n', 'utf8')
  }

  return { path: configPath, old: oldVersion, new: newVersion }
}

// Update Cargo.toml
function updateCargoToml(newVersion, dryRun) {
  const cargoPath = path.join(process.cwd(), 'src-tauri', 'Cargo.toml')
  let content = fs.readFileSync(cargoPath, 'utf8')

  const versionRegex = /^version\s*=\s*"([^"]+)"/m
  const match = content.match(versionRegex)

  if (!match) {
    throw new Error('Could not find version in Cargo.toml')
  }

  const oldVersion = match[1]
  const newContent = content.replace(versionRegex, `version = "${newVersion}"`)

  if (!dryRun) {
    fs.writeFileSync(cargoPath, newContent, 'utf8')
  }

  return { path: cargoPath, old: oldVersion, new: newVersion }
}

// Main execution
function main() {
  try {
    const options = parseArgs()

    if (!options.type) {
      console.error('Error: --type is required (patch|minor|major)')
      process.exit(1)
    }

    if (!['patch', 'minor', 'major'].includes(options.type)) {
      console.error(`Error: Invalid bump type '${options.type}'. Must be patch, minor, or major.`)
      process.exit(1)
    }

    if (!['stable', 'alpha', 'beta', 'rc'].includes(options.channel)) {
      console.error(
        `Error: Invalid channel '${options.channel}'. Must be stable, alpha, beta, or rc.`
      )
      process.exit(1)
    }

    // Read current version from package.json
    const pkgPath = path.join(process.cwd(), 'package.json')
    const pkgContent = fs.readFileSync(pkgPath, 'utf8')
    const pkg = JSON.parse(pkgContent)
    const currentVersion = pkg.version

    // Calculate new version
    const newVersion = bumpVersion(currentVersion, options.type, options.channel)

    console.log('\n📦 Version Bump Summary\n')
    console.log(`Current version: ${currentVersion}`)
    console.log(`Bump type:       ${options.type}`)
    console.log(`Channel:         ${options.channel}`)
    console.log(`New version:     ${newVersion}`)

    if (options.dryRun) {
      console.log('\n🔍 DRY RUN - No files will be modified\n')
    } else {
      console.log('')
    }

    // Update files
    const packageResult = updatePackageJson(newVersion, options.dryRun)
    console.log(`${options.dryRun ? '[DRY RUN]' : '✓'} ${packageResult.path}`)
    console.log(`  ${packageResult.old} → ${packageResult.new}`)

    const tauriResult = updateTauriConfig(newVersion, options.dryRun)
    console.log(`${options.dryRun ? '[DRY RUN]' : '✓'} ${tauriResult.path}`)
    console.log(`  ${tauriResult.old} → ${tauriResult.new}`)

    const cargoResult = updateCargoToml(newVersion, options.dryRun)
    console.log(`${options.dryRun ? '[DRY RUN]' : '✓'} ${cargoResult.path}`)
    console.log(`  ${cargoResult.old} → ${cargoResult.new}`)

    if (!options.dryRun) {
      console.log('\n✨ Version bump complete!\n')
      console.log('Next steps:')
      console.log('  1. Review the changes: git diff')
      console.log(
        '  2. Commit the changes: git add . && git commit -m "chore: bump version to ' +
          newVersion +
          '"'
      )
      console.log('  3. Push and trigger release workflow\n')
    }

    // Output for GitHub Actions
    if (process.env.GITHUB_OUTPUT) {
      fs.appendFileSync(process.env.GITHUB_OUTPUT, `version=${newVersion}\n`)
    }
  } catch (error) {
    console.error(`\n❌ Error: ${error.message}\n`)
    process.exit(1)
  }
}

main()
