const fs = require('fs');
const path = require('path');

// Simple JSON Schema validator (basic implementation)
function validateSchema(data, schema) {
  const errors = [];
  
  // Check type
  if (schema.type && typeof data !== schema.type && !(schema.type === 'array' && Array.isArray(data))) {
    if (schema.type === 'object' && typeof data !== 'object') {
      errors.push(`Expected type ${schema.type}, got ${typeof data}`);
    }
  }
  
  // Check required properties for objects
  if (schema.type === 'object' && schema.required) {
    schema.required.forEach(prop => {
      if (!(prop in data)) {
        errors.push(`Missing required property: ${prop}`);
      }
    });
  }
  
  // Check properties
  if (schema.type === 'object' && schema.properties) {
    Object.keys(data).forEach(key => {
      if (schema.properties[key]) {
        const propErrors = validateSchema(data[key], schema.properties[key]);
        errors.push(...propErrors.map(e => `${key}.${e}`));
      }
    });
  }
  
  // Check pattern
  if (schema.pattern && typeof data === 'string') {
    const regex = new RegExp(schema.pattern);
    if (!regex.test(data)) {
      errors.push(`String does not match pattern ${schema.pattern}: "${data}"`);
    }
  }
  
  // Check enum
  if (schema.enum && !schema.enum.includes(data)) {
    errors.push(`Value "${data}" not in enum: [${schema.enum.join(', ')}]`);
  }
  
  // Check minimum
  if (schema.minimum !== undefined && typeof data === 'number' && data < schema.minimum) {
    errors.push(`Number ${data} is less than minimum ${schema.minimum}`);
  }
  
  return errors;
}

console.log('='.repeat(60));
console.log('JSON Schema Validation Tests');
console.log('='.repeat(60));

const schemasDir = path.join(__dirname);
const fixturesDir = path.join(__dirname, 'test-fixtures');

// Test 1: Validate cache-metadata v2.1
console.log('\n1. Testing cache-metadata.schema.json with v2.1 fixture...');
try {
  const schema = JSON.parse(fs.readFileSync(path.join(schemasDir, 'cache-metadata.schema.json'), 'utf8'));
  const data = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'cache_metadata_v2.1_valid.json'), 'utf8'));
  
  const errors = validateSchema(data, schema);
  if (errors.length === 0) {
    console.log('   ✅ PASS: v2.1 cache metadata is valid');
  } else {
    console.log('   ❌ FAIL: Validation errors:');
    errors.forEach(e => console.log(`      - ${e}`));
  }
} catch (err) {
  console.log(`   ❌ ERROR: ${err.message}`);
}

// Test 2: Validate cache-metadata v2.0
console.log('\n2. Testing cache-metadata.schema.json with v2.0 fixture...');
try {
  const schema = JSON.parse(fs.readFileSync(path.join(schemasDir, 'cache-metadata.schema.json'), 'utf8'));
  const data = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'cache_metadata_v2.0_valid.json'), 'utf8'));
  
  const errors = validateSchema(data, schema);
  if (errors.length === 0) {
    console.log('   ✅ PASS: v2.0 cache metadata is valid');
  } else {
    console.log('   ❌ FAIL: Validation errors:');
    errors.forEach(e => console.log(`      - ${e}`));
  }
} catch (err) {
  console.log(`   ❌ ERROR: ${err.message}`);
}

// Test 3: Validate read_context request
console.log('\n3. Testing MCP read_context request...');
try {
  const data = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'read_context_valid.json'), 'utf8'));
  
  // Manual validation for read_context
  const errors = [];
  if (!data.params || !data.params.query) {
    errors.push('Missing required field: params.query');
  }
  if (data.params && data.params.query && data.params.query.length < 1) {
    errors.push('query must have minimum length 1');
  }
  if (data.params && data.params.limit && (data.params.limit < 1 || data.params.limit > 50)) {
    errors.push('limit must be between 1 and 50');
  }
  
  if (errors.length === 0) {
    console.log('   ✅ PASS: read_context request is valid');
  } else {
    console.log('   ❌ FAIL: Validation errors:');
    errors.forEach(e => console.log(`      - ${e}`));
  }
} catch (err) {
  console.log(`   ❌ ERROR: ${err.message}`);
}

// Test 4: Schema files are valid JSON
console.log('\n4. Checking all schema files are valid JSON...');
const schemaFiles = ['mcp-tools.schema.json', 'cache-metadata.schema.json', 'security-patterns.schema.json'];
schemaFiles.forEach(file => {
  try {
    JSON.parse(fs.readFileSync(path.join(schemasDir, file), 'utf8'));
    console.log(`   ✅ ${file}: Valid JSON`);
  } catch (err) {
    console.log(`   ❌ ${file}: ${err.message}`);
  }
});

console.log('\n' + '='.repeat(60));
console.log('Validation Complete');
console.log('='.repeat(60));
console.log('\nNote: For full JSON Schema validation, install ajv-cli:');
console.log('  npm install -g ajv-cli');
console.log('  ajv validate -s cache-metadata.schema.json -d test-fixtures/*.json');
