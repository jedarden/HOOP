#!/usr/bin/env node
/**
 * Generate TypeScript types from JSON Schema files.
 *
 * This script reads all JSON Schema files from hoop-schema/schemas/
 * and generates TypeScript types using json-schema-to-typescript.
 * Output is written to hoop-schema/ts/index.d.ts
 */

import { readFileSync, readdirSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { compile } from 'json-schema-to-typescript';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Paths
const schemasDir = join(__dirname, '../schemas');
const outputDir = join(__dirname, '../ts');
const outputFile = join(outputDir, 'index.d.ts');

// Schema files to process (order matters for dependencies)
const schemaOrder = [
  'worker_liveness.json',
  'worker_display_state.json',
  'worker_metadata.json',
  'worker_data.json',
  'message_usage.json',
  'bead_data.json',
  'bead.json',
  'bead_created_by_hoop.json',
  'bead_blockers_response.json',
  'session_kind.json',
  'session_message.json',
  'parsed_session.json',
  'conversation_data.json',
  'streaming_content.json',
  'ws_message.json',
  'ui_state.json',
  'capacity_limits.json',
  'capacity_usage.json',
  'capacity_account.json',
  'workspace_entry.json',
  'project_entry.json',
  'projects_registry.json',
  'agent_config.json',
  'backup_config.json',
  'ui_config.json',
  'voice_config.json',
  'pricing_config.json',
  'cost_bucket.json',
  'hoop_config.json',
  'audit_row.json',
  'stitch.json',
  'stitch_bead.json',
  'stitch_link.json',
  'stitch_message.json',
  'stitch_preview.json',
  'stitch_template.json',
  'pattern.json',
  'pattern_member.json',
  'pattern_query.json',
  'reflection_ledger.json',
  'config_error.json',
  'project_config_status.json',
  'cross_workspace_blocker.json',
  'template_field.json',
  'presence.json',
  'dictated_note.json',
  'redaction_policy.json',
  'script_manifest.json',
  'secret_pattern.json',
  'unknown_events_response.json',
  'unknown_event_samples_response.json',
  'debug_state.json',
  'morning_brief_config.json',
  'stuck_detector_config.json',
  'embedding_config.json',
  'voice_config.json',
];

/**
 * Inline all $ref references in a schema by replacing them with the referenced schema content.
 * This creates a fully expanded schema with no external references.
 */
function inlineRefs(schema, allSchemas, visited = new Set()) {
  if (typeof schema !== 'object' || schema === null) {
    return schema;
  }

  if (Array.isArray(schema)) {
    return schema.map(item => inlineRefs(item, allSchemas, visited));
  }

  // Check for $ref at this level
  if (schema.$ref && typeof schema.$ref === 'string') {
    const refPath = schema.$ref;
    const filename = refPath.includes('/') ? refPath.split('/').pop() : refPath;

    if (allSchemas[filename] && !visited.has(filename)) {
      visited.add(filename);
      // Clone and recursively inline refs in the referenced schema
      const referenced = inlineRefs({ ...allSchemas[filename] }, allSchemas, visited);
      // Merge the referenced schema, excluding metadata keys
      const result = { ...referenced };
      delete result.$schema;
      delete result.$id;
      delete result.title;
      delete result.description;
      // Also merge any additional properties from the original schema
      const { $ref, ...additionalProps } = schema;
      return { ...result, ...additionalProps };
    }
  }

  // Recursively inline refs in all properties
  const result = {};
  for (const [key, value] of Object.entries(schema)) {
    if (key !== '$ref') { // Skip $ref as we've already handled it
      result[key] = inlineRefs(value, allSchemas, visited);
    }
  }
  return result;
}

/**
 * Convert JSON Schema to TypeScript type definition.
 */
async function schemaToType(filename, schemaContent, allSchemas) {
  let schema;
  try {
    schema = JSON.parse(schemaContent);
  } catch (err) {
    throw new Error(`Failed to parse ${filename}: ${err.message}`);
  }

  // Extract the title for the type name
  const title = schema.title;
  if (!title) {
    throw new Error(`${filename} missing required "title" property`);
  }

  // Inline all $ref references
  const inlinedSchema = inlineRefs(schema, allSchemas);

  // Clean up schema for json-schema-to-typescript
  const cleanSchema = { ...inlinedSchema };
  delete cleanSchema.schema_version;
  delete cleanSchema.$schema;
  delete cleanSchema.$id;

  try {
    const ts = await compile(cleanSchema, title, {
      bannerComment: '',
      unreachableDefinitions: false,
    });
    return ts;
  } catch (err) {
    throw new Error(`Failed to compile ${filename}: ${err.message}`);
  }
}

async function main() {
  // Read all schemas into memory
  const allSchemas = {};
  const schemaFiles = readdirSync(schemasDir).filter(f => f.endsWith('.json'));

  for (const file of schemaFiles) {
    const content = readFileSync(join(schemasDir, file), 'utf-8');
    try {
      allSchemas[file] = JSON.parse(content);
    } catch (err) {
      console.error(`Warning: Failed to parse ${file}, skipping`);
    }
  }

  // Generate TypeScript types in dependency order
  const typeDefinitions = [];
  const errors = [];

  for (const filename of schemaOrder) {
    if (!allSchemas[filename]) {
      console.warn(`Warning: ${filename} not found, skipping`);
      continue;
    }

    try {
      const content = readFileSync(join(schemasDir, filename), 'utf-8');
      const ts = await schemaToType(filename, content, allSchemas);
      typeDefinitions.push(ts);
      console.log(`✓ Generated type for ${filename}`);
    } catch (err) {
      errors.push(`${filename}: ${err.message}`);
      console.error(`✗ Failed to generate type for ${filename}: ${err.message}`);
    }
  }

  if (errors.length > 0) {
    console.error('\nErrors encountered:');
    errors.forEach(e => console.error(`  - ${e}`));
    process.exit(1);
  }

  // Write output file
  const header = `/**
 * AUTO-GENERATED TypeScript types from JSON Schema
 *
 * Source: hoop-schema/schemas/
 * Generated by: hoop-schema/scripts/codegen-types.mjs
 * Do not edit manually - regenerate with: pnpm --filter hoop-schema run codegen
 *
 * Schema version: 1.0.0
 */

`;

  const output = header + typeDefinitions.join('\n\n');

  // Ensure output directory exists
  mkdirSync(outputDir, { recursive: true });

  writeFileSync(outputFile, output, 'utf-8');
  console.log(`\n✓ Generated ${outputFile}`);
  console.log(`  Processed ${typeDefinitions.length} schemas`);
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
