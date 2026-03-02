#!/usr/bin/env node

/**
 * OpenClaw CLI
 * 
 * Quick commands for OpenClaw State Engine:
 * - init: Bootstrap a new project
 * - health: Check state engine status
 * - generate-key: Create idempotency keys
 * - run-example: Quick start examples
 */

import { Command } from 'commander';
import chalk from 'chalk';
import fs from 'fs/promises';
import path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const program = new Command();

const DEFAULT_ENGINE_URL = 'http://127.0.0.1:3030';

// ASCII art banner
const banner = `
${chalk.cyan('╔══════════════════════════════════════════╗')}
${chalk.cyan('║')}     🔮 ${chalk.bold('OpenClaw State Engine CLI')} 🔮    ${chalk.cyan('║')}
${chalk.cyan('║')}      Persistent state for AI agents      ${chalk.cyan('║')}
${chalk.cyan('╚══════════════════════════════════════════╝')}
`;

console.log(banner);

program
  .name('openclaw')
  .description('CLI for OpenClaw State Engine')
  .version('0.1.0');

// Health check command
program
  .command('health')
  .description('Check state engine health')
  .option('-u, --url <url>', 'State engine URL', DEFAULT_ENGINE_URL)
  .action(async (options) => {
    console.log(chalk.blue('🔍 Checking state engine health...\n'));
    
    try {
      const response = await fetch(`${options.url}/health`);
      if (response.ok) {
        console.log(chalk.green('✅ State engine is healthy'));
        console.log(chalk.gray(`   URL: ${options.url}`));
      } else {
        console.log(chalk.red(`❌ State engine returned ${response.status}`));
      }
    } catch (error) {
      console.log(chalk.red('❌ State engine is not responding'));
      console.log(chalk.gray(`   URL: ${options.url}`));
      console.log(chalk.yellow('\n💡 Start the engine:'));
      console.log(chalk.gray('   cd state-engine && cargo run -- serve'));
    }
  });

// Generate idempotency key
program
  .command('generate-key')
  .alias('key')
  .description('Generate an idempotency key')
  .option('-p, --prefix <prefix>', 'Key prefix', 'task')
  .argument('[name]', 'Descriptive name for the key')
  .action((name, options) => {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    const keyName = name || `${options.prefix}`;
    const key = `${keyName}-${timestamp}-${random}`;
    
    console.log(chalk.green('🔑 Generated idempotency key:'));
    console.log(chalk.white(`   ${key}`));
    console.log(chalk.gray('\n   Use this key with execute_tool_idempotent()'));
    console.log(chalk.gray('   Same key = same result, always'));
  });

// Initialize new project
program
  .command('init')
  .description('Bootstrap a new OpenClaw project')
  .argument('[project-name]', 'Project directory name', 'my-openclaw-project')
  .option('-t, --template <type>', 'Template: basic, email, scraper', 'basic')
  .action(async (projectName, options) => {
    const projectPath = path.resolve(projectName);
    
    console.log(chalk.blue(`📦 Creating project: ${projectName}\n`));
    
    try {
      // Create directory
      await fs.mkdir(projectPath, { recursive: true });
      await fs.mkdir(path.join(projectPath, 'src'), { recursive: true });
      
      // Create package.json
      const packageJson = {
        name: projectName,
        version: '0.1.0',
        type: 'module',
        scripts: {
          start: 'node src/index.js',
          dev: 'node --watch src/index.js'
        },
        dependencies: {
          '@openclaw/ts-client': 'file:../../ts-client'
        }
      };
      await fs.writeFile(
        path.join(projectPath, 'package.json'),
        JSON.stringify(packageJson, null, 2)
      );
      
      // Create index.js based on template
      const indexContent = generateTemplate(options.template, projectName);
      await fs.writeFile(path.join(projectPath, 'src', 'index.js'), indexContent);
      
      // Create README
      const readmeContent = generateReadme(projectName, options.template);
      await fs.writeFile(path.join(projectPath, 'README.md'), readmeContent);
      
      // Create .env.example
      const envContent = `STATE_ENGINE_URL=http://127.0.0.1:3030\nUSER_NAME=${projectName}-agent`;
      await fs.writeFile(path.join(projectPath, '.env.example'), envContent);
      
      console.log(chalk.green('✅ Project created successfully!\n'));
      console.log(chalk.white('Next steps:'));
      console.log(chalk.gray(`   cd ${projectName}`));
      console.log(chalk.gray('   npm install'));
      console.log(chalk.gray('   cp .env.example .env'));
      console.log(chalk.gray('   npm start'));
      console.log(chalk.gray('\nMake sure the state engine is running:'));
      console.log(chalk.gray('   cd open-engine/state-engine && cargo run -- serve'));
    } catch (error) {
      console.error(chalk.red(`❌ Error creating project: ${error.message}`));
    }
  });

// Run example command
program
  .command('run-example')
  .alias('example')
  .description('Quick start an example')
  .argument('<name>', 'Example name: email-agent, web-scraper, dashboard')
  .action(async (name) => {
    const validExamples = ['email-agent', 'web-scraper', 'dashboard'];
    
    if (!validExamples.includes(name)) {
      console.log(chalk.red(`❌ Unknown example: ${name}`));
      console.log(chalk.yellow(`Available: ${validExamples.join(', ')}`));
      return;
    }
    
    console.log(chalk.blue(`🚀 Starting ${name} example...\n`));
    
    try {
      const examplePath = path.join(process.cwd(), '..', '..', 'examples', name);
      console.log(chalk.gray(`   cd ${examplePath}`));
      console.log(chalk.gray('   npm install'));
      console.log(chalk.gray('   npm start\n'));
      
      // Check if engine is running
      try {
        await fetch(`${DEFAULT_ENGINE_URL}/health`);
      } catch {
        console.log(chalk.yellow('⚠️  State engine not detected. Starting it first...\n'));
      }
      
      console.log(chalk.green('✅ Ready to run!'));
    } catch (error) {
      console.error(chalk.red(`❌ Error: ${error.message}`));
    }
  });

// Quickstart command (combine health check + init)
program
  .command('quickstart')
  .description('One-command setup: check health and show next steps')
  .action(async () => {
    console.log(chalk.cyan('\n🔮 OpenClaw Quickstart\n'));
    
    // Check health
    let engineReady = false;
    try {
      const response = await fetch(`${DEFAULT_ENGINE_URL}/health`);
      if (response.ok) {
        engineReady = true;
        console.log(chalk.green('✅ State engine is running'));
      }
    } catch {
      console.log(chalk.red('❌ State engine not running'));
    }
    
    if (!engineReady) {
      console.log(chalk.yellow('\n📋 To start the state engine:'));
      console.log(chalk.white('   1. cd open-engine/state-engine'));
      console.log(chalk.white('   2. cargo run -- serve'));
      console.log(chalk.gray('   (Leave this running in a separate terminal)\n'));
    } else {
      console.log(chalk.green('\n✅ Everything is ready!'));
      console.log(chalk.white('\nNext steps:'));
      console.log(chalk.gray('   openclaw init my-project'));
      console.log(chalk.gray('   cd my-project'));
      console.log(chalk.gray('   npm install && npm start'));
    }
    
    console.log(chalk.cyan('\n📚 Useful commands:'));
    console.log(chalk.gray('   openclaw health       - Check if engine is running'));
    console.log(chalk.gray('   openclaw generate-key - Create idempotency keys'));
    console.log(chalk.gray('   openclaw init         - Create a new project'));
    console.log(chalk.gray('   openclaw run-example  - Run demo examples\n'));
  });

// Generate template code
function generateTemplate(type, projectName) {
  const base = `import { StateEngineClient } from '@openclaw/ts-client';

const client = new StateEngineClient({
  url: process.env.STATE_ENGINE_URL || 'http://127.0.0.1:3030'
});

async function main() {
  console.log('🔮 Starting ${projectName}...');
  
  // Create user
  const user = await client.createUser('${projectName}-agent');
  console.log('👤 User:', user.id);
  
  // Create session
  const session = await client.createSession(user.id);
  console.log('📂 Session:', session.id);
`;

  if (type === 'email') {
    return base + `
  // Example: Send an email with idempotency
  const goal = await client.createGoal(user.id, session.id, 'Send welcome email');
  
  const result = await client.executeToolIdempotent(
    user.id, session.id, goal.id,
    'send_email',
    { to: 'user@example.com', subject: 'Welcome!' },
    'welcome-email-001'
  );
  
  console.log('✅ Email execution:', result.id);
  console.log('   Status:', result.status);
}

main().catch(console.error);
`;
  }

  if (type === 'scraper') {
    return base + `
  // Example: Scrape a URL with idempotency
  const goal = await client.createGoal(user.id, session.id, 'Scrape example.com');
  
  const result = await client.executeToolIdempotent(
    user.id, session.id, goal.id,
    'scrape_url',
    { url: 'https://example.com' },
    'scrape-example-001'
  );
  
  console.log('✅ Scrape execution:', result.id);
  console.log('   Same URL will return cached result');
}

main().catch(console.error);
`;
  }

  return base + `
  // Create a goal
  const goal = await client.createGoal(user.id, session.id, 'Demo task');
  console.log('🎯 Goal:', goal.id);
  
  // Execute a tool with idempotency
  const result = await client.executeToolIdempotent(
    user.id, session.id, goal.id,
    'demo_tool',
    { message: 'Hello from ${projectName}!' },
    'demo-execution-001'
  );
  
  console.log('✅ Execution:', result.id);
  console.log('   Status:', result.status);
}

main().catch(console.error);
`;
}

// Generate README
function generateReadme(projectName, template) {
  return `# ${projectName}

Created with OpenClaw CLI 🔮

## Quick Start

1. Install dependencies:
   \`\`\`bash
   npm install
   \`\`\`

2. Start the state engine (in another terminal):
   \`\`\`bash
   cd open-engine/state-engine
   cargo run -- serve
   \`\`\`

3. Run your project:
   \`\`\`bash
   npm start
   \`\`\`

## Features

- ✅ Durable state that survives crashes
- ✅ Idempotent tool execution (no duplicates)
- ✅ Full audit trail

## Template: ${template}

This project uses the "${template}" template as a starting point.

## Learn More

- [OpenClaw State Engine Docs](https://github.com/3lvin-Kc/open-engine)
- [API Reference](../docs/API_VERSION.md)
`;
}

program.parse();
