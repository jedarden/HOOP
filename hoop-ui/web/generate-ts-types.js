const fs = require('fs');
const path = require('path');
const { compile } = require('json-schema-to-typescript');

const schemasDir = path.join(__dirname, '../../hoop-schema/schemas');
const outputDir = path.join(__dirname, '../../hoop-schema/ts');

// Ensure output directory exists
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Read all JSON schema files
const schemaFiles = fs.readdirSync(schemasDir)
  .filter(f => f.endsWith('.json'))
  .sort();

async function generateSchemaTypes() {
  let allTypes = '';
  const imports = new Set();

  for (const file of schemaFiles) {
    const schemaPath = path.join(schemasDir, file);
    const schemaContent = fs.readFileSync(schemaPath, 'utf8');
    const schema = JSON.parse(schemaContent);

    // Extract the type name from the filename (e.g., "bead.json" -> "Bead")
    const typeName = file.replace('.json', '')
      .split('_')
      .map((part, i) => part.charAt(0).toUpperCase() + part.slice(1))
      .join('');

    try {
      const ts = await compile(schema, typeName, {
        bannerComment: '',
        unreachableDefinitions: false,
      });
      allTypes += `\n// ===== ${file} =====\n\n`;
      allTypes += ts + '\n';
    } catch (e) {
      console.error(`Failed to compile ${file}:`, e.message);
    }
  }

  // Write the combined types file
  fs.writeFileSync(path.join(outputDir, 'index.ts'), allTypes);
  console.log(`Generated TypeScript types for ${schemaFiles.length} schemas`);
}

generateSchemaTypes().catch(console.error);
