/**
 * Automatic Service Generator
 *
 * Scans Rust backend endpoints and generates TypeScript service classes.
 *
 * Supported HTTP Methods:
 * - GET: Generates methods with query params
 * - POST: Generates methods with request body
 * - DELETE: Generates methods with optional request body
 * - PUT: Generates methods with request body
 * - PATCH: Generates methods with request body
 *
 * Special Cases:
 * - Multipart handlers → FormData parameter
 * - download_file handler → Blob response type
 *
 * Usage:
 * - npm run dev (automatic)
 * - npm run generate:services (manual)
 */

import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SERVER_PATH = join(__dirname, '../../server/src');
const SERVICES_PATH = join(__dirname, '../src/lib/services');

// Extract routes from main.rs
function extractRoutes() {
	const mainRs = readFileSync(join(SERVER_PATH, 'main.rs'), 'utf-8');
	// Match .route("path", method(handler)) where method can be get, post, delete, put, patch, etc.
	const routeRegex = /\.route\("([^"]+)",\s*(get|post|delete|put|patch)\((\w+)\)/g;
	const routes = [];
	let match;

	while ((match = routeRegex.exec(mainRs)) !== null) {
		routes.push({
			path: match[1],
			method: match[2].toUpperCase(), // GET, POST, DELETE, etc.
			handlerName: match[3]
		});
	}

	return routes;
}

// Extract request types from models/mod.rs
function extractRequestTypes() {
	const modelsRs = readFileSync(join(SERVER_PATH, 'models/mod.rs'), 'utf-8');
	const structRegex = /pub struct (\w+)\s*\{([^}]+)\}/gs;
	const types = {};
	let match;

	while ((match = structRegex.exec(modelsRs)) !== null) {
		const structName = match[1];
		const fieldsText = match[2];
		const fields = [];

		// Updated regex to capture generic types like Vec<Type> or Option<Vec<Type>>
		const fieldRegex = /pub (\w+):\s*(Option<)?(Vec<\w+>|\w+)>?,?/g;
		let fieldMatch;

		while ((fieldMatch = fieldRegex.exec(fieldsText)) !== null) {
			fields.push({
				name: fieldMatch[1],
				optional: !!fieldMatch[2],
				rustType: fieldMatch[3]
			});
		}

		types[structName] = fields;
	}

	return types;
}

// Map Rust types to TypeScript types
function rustToTsType(rustType) {
	// Handle Vec<Type> -> Type[]
	if (rustType.startsWith('Vec<') && rustType.endsWith('>')) {
		const innerType = rustType.slice(4, -1);
		const convertedInner = rustToTsType(innerType);
		return `${convertedInner}[]`;
	}

	const typeMap = {
		String: 'string',
		i32: 'number',
		i64: 'number',
		f32: 'number',
		f64: 'number',
		bool: 'boolean',
		usize: 'number'
	};
	return typeMap[rustType] || rustType;
}

// Convert Rust return type to TypeScript type
function rustReturnTypeToTs(rustType) {
	if (!rustType) return 'unknown';

	// Handle Vec<Type> -> Type[]
	if (rustType.startsWith('Vec<') && rustType.endsWith('>')) {
		const innerType = rustType.slice(4, -1);
		// Recursively convert the inner type
		const convertedInner = rustReturnTypeToTs(innerType);
		return `${convertedInner}[]`;
	}

	// Handle String -> string
	if (rustType === 'String') return 'string';

	// Handle other primitives
	const typeMap = {
		i32: 'number',
		i64: 'number',
		f32: 'number',
		f64: 'number',
		bool: 'boolean',
		usize: 'number'
	};

	return typeMap[rustType] || rustType;
}

// Find which request type and return type each handler uses
function mapHandlersToTypes() {
	const fileRs = readFileSync(join(SERVER_PATH, 'endpoints/file.rs'), 'utf-8');
	const folderRs = readFileSync(join(SERVER_PATH, 'endpoints/folder.rs'), 'utf-8');
	const mapping = {};
	const returnTypes = {};

	// Split into functions
	const processFile = (content) => {
		// Match pub async fn function_name
		const functionMatches = content.matchAll(/pub async fn (\w+)/g);

		for (const funcMatch of functionMatches) {
			const funcName = funcMatch[1];
			const funcStart = funcMatch.index;

			// Look ahead from function start to find Json<Type> or Multipart
			const lookAhead = content.substring(funcStart, funcStart + 500);

			// Try to find Multipart first (takes precedence)
			const multipartMatch = lookAhead.match(/mut multipart:\s*Multipart/i);
			if (multipartMatch) {
				mapping[funcName] = 'FormData';
				continue;
			}

			// Try to find Json<Type> in function parameters (before -> return)
			// Match only parameters, not return types
			const paramsSection = lookAhead.split('->')[0]; // Get everything before return type
			const jsonMatch = paramsSection.match(/Json<([^>]+)>/);
			if (jsonMatch) {
				// Extract clean type name
				const typeName = jsonMatch[1].trim();
				// Handle module paths like crate::models::TypeName
				const cleanType = typeName.includes('::') ? typeName.split('::').pop() : typeName;
				mapping[funcName] = cleanType;
			}

			// Extract return type from Result<Json<T>, StatusCode>
			// Need to handle nested angle brackets like Vec<Type>
			const returnMatch = lookAhead.match(/->\s*Result<Json<(.+?)>,\s*StatusCode>/);
			if (returnMatch) {
				let returnType = returnMatch[1].trim();
				// Store as-is, will convert later
				returnTypes[funcName] = returnType;
			} else if (lookAhead.match(/->\s*Result<Response/)) {
				// For download_file which returns Response -> Blob
				returnTypes[funcName] = 'Blob';
			}
		}
	};

	processFile(fileRs);
	processFile(folderRs);

	return { requestTypes: mapping, returnTypes };
}

// Classify endpoints into file or folder services
function classifyEndpoint(handlerName) {
	const fileHandlers = [
		'upload_file',
		'download_file',
		'delete_file',
		'get_list_file_and_folder',
		'list_trash',
		'restore_file',
		'empty_trash'
	];
	const folderHandlers = [
		'create_folder',
		'rename_folder',
		'search_files',
		'sorted_list_file_and_folder'
	];

	if (fileHandlers.includes(handlerName)) return 'file';
	if (folderHandlers.includes(handlerName)) return 'folder';
	return 'file'; // default
}

// Generate TypeScript interfaces
function generateInterfaces(types) {
	let code = '// Auto-generated types from Rust backend\n\n';

	for (const [typeName, fields] of Object.entries(types)) {
		code += `export interface ${typeName} {\n`;
		for (const field of fields) {
			const tsType = rustToTsType(field.rustType);
			const optional = field.optional ? '?' : '';
			code += `	${field.name}${optional}: ${tsType};\n`;
		}
		code += '}\n\n';
	}

	return code;
}

// Convert handler name to method name (snake_case to camelCase)
function toCamelCase(snakeCase) {
	return snakeCase.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

// Generate service methods
function generateServiceMethods(routes, handlerTypes, returnTypes) {
	const fileServiceMethods = [];
	const folderServiceMethods = [];

	for (const route of routes) {
		const { path, method, handlerName } = route;
		const requestType = handlerTypes[handlerName];
		const rawReturnType = returnTypes[handlerName] || 'unknown';
		const returnType = rustReturnTypeToTs(rawReturnType);
		const methodName = toCamelCase(handlerName);
		const category = classifyEndpoint(handlerName);
		const httpMethod = method.toLowerCase(); // 'get', 'post', 'delete', etc.

		let methodCode;

		// Handle FormData uploads
		if (requestType === 'FormData') {
			methodCode = `	async ${methodName}(formData: FormData) {
		return this.${httpMethod}<${returnType}>('${path}', formData, {
			headers: { 'Content-Type': 'multipart/form-data' }
		});
	}`;
		}
		// Handle file downloads
		else if (handlerName === 'download_file') {
			methodCode = `	async ${methodName}(request: ${requestType}) {
		return this.${httpMethod}<Blob>('${path}', request, { responseType: 'blob' });
	}`;
		}
		// Handle GET requests (no request body)
		else if (method === 'GET') {
			if (requestType && requestType !== 'unknown') {
				methodCode = `	async ${methodName}(params?: ${requestType}) {
		return this.get<${returnType}>('${path}', { params });
	}`;
			} else {
				methodCode = `	async ${methodName}() {
		return this.get<${returnType}>('${path}');
	}`;
			}
		}
		// Handle DELETE requests
		else if (method === 'DELETE') {
			if (requestType) {
				methodCode = `	async ${methodName}(request: ${requestType}) {
		return this.delete<${returnType}>('${path}', { data: request });
	}`;
			} else {
				methodCode = `	async ${methodName}() {
		return this.delete<${returnType}>('${path}');
	}`;
			}
		}
		// Handle POST/PUT/PATCH with request body
		else {
			if (requestType && requestType !== 'unknown') {
				methodCode = `	async ${methodName}(request: ${requestType}) {
		return this.${httpMethod}<${returnType}>('${path}', request);
	}`;
			} else {
				// POST without body (like empty_trash)
				methodCode = `	async ${methodName}() {
		return this.${httpMethod}<${returnType}>('${path}');
	}`;
			}
		}

		if (category === 'file') {
			fileServiceMethods.push(methodCode);
		} else {
			folderServiceMethods.push(methodCode);
		}
	}

	return { fileServiceMethods, folderServiceMethods };
}

// Main execution
function main() {
	console.log('🔍 Scanning backend endpoints...');

	const routes = extractRoutes();
	const types = extractRequestTypes();
	const { requestTypes: handlerTypes, returnTypes } = mapHandlersToTypes();

	console.log(`✅ Found ${routes.length} endpoints`);
	console.log(`✅ Found ${Object.keys(types).length} request types`);
	console.log(
		'📋 Routes:',
		routes.map((r) => `${r.method} ${r.path} -> ${r.handlerName}`).join('\n       ')
	);
	console.log('📋 Handler to Type mapping:', handlerTypes);
	console.log('📋 Return Types:', returnTypes);

	// Generate types file
	const typesCode = generateInterfaces(types);
	writeFileSync(join(SERVICES_PATH, 'types.ts'), typesCode);
	console.log('✅ Generated types.ts');

	// Generate service methods
	const { fileServiceMethods, folderServiceMethods } = generateServiceMethods(
		routes,
		handlerTypes,
		returnTypes
	);

	// Collect used types for each service
	const fileServiceTypes = new Set();
	const folderServiceTypes = new Set();

	for (const route of routes) {
		const { handlerName } = route;
		const requestType = handlerTypes[handlerName];
		const rawReturnType = returnTypes[handlerName];
		const category = classifyEndpoint(handlerName);

		// Add request types
		if (requestType && requestType !== 'FormData' && requestType !== 'unknown') {
			// Filter out invalid types (Vec<...>, primitives)
			if (
				!requestType.startsWith('Vec<') &&
				!['String', 'i32', 'i64', 'f32', 'f64', 'bool'].includes(requestType)
			) {
				if (category === 'file') {
					fileServiceTypes.add(requestType);
				} else {
					folderServiceTypes.add(requestType);
				}
			}
		}

		// Add return types (extract from Vec<Type> or Type)
		if (rawReturnType) {
			let typeToImport = rawReturnType;
			// Extract inner type from Vec<Type>
			if (typeToImport.startsWith('Vec<') && typeToImport.endsWith('>')) {
				typeToImport = typeToImport.slice(4, -1);
			}
			// Only import custom types, not primitives
			if (
				typeToImport &&
				!['String', 'i32', 'i64', 'f32', 'f64', 'bool', 'Blob', 'unknown'].includes(typeToImport)
			) {
				if (category === 'file') {
					fileServiceTypes.add(typeToImport);
				} else {
					folderServiceTypes.add(typeToImport);
				}
			}
		}
	}

	// Update FileService
	const fileServiceImports = Array.from(fileServiceTypes).join(',\n\t');
	const fileServiceCode = `import { ServiceBase } from './ServiceBase';
import type {
	${fileServiceImports}
} from './types';

export class FileService extends ServiceBase {
	constructor() {
		super();
	}

${fileServiceMethods.join('\n\n')}
}
`;

	writeFileSync(join(SERVICES_PATH, 'FileService.ts'), fileServiceCode);
	console.log('✅ Updated FileService.ts');

	// Update FolderService
	const folderServiceImports = Array.from(folderServiceTypes).join(',\n\t');
	const folderServiceCode = `import { ServiceBase } from './ServiceBase';
import type {
	${folderServiceImports}
} from './types';

export class FolderService extends ServiceBase {
	constructor() {
		super();
	}

${folderServiceMethods.join('\n\n')}
}
`;

	writeFileSync(join(SERVICES_PATH, 'FolderService.ts'), folderServiceCode);
	console.log('✅ Updated FolderService.ts');

	console.log('🎉 Service generation complete!');
}

main();
